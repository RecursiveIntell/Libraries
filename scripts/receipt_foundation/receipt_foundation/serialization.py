"""Strict JSON framing with explicit, observable historical stream recovery.

Numbers retain their exact lexical spelling. Structural hashes are NOT JCS and
never replace raw-byte identity. Recovery does not skip malformed islands.
"""
from __future__ import annotations
import dataclasses, hashlib, json, pathlib, re
from typing import Any
from .common import FoundationError, Limits, canonical_json, digest

@dataclasses.dataclass(frozen=True)
class Number:
    raw: str

@dataclasses.dataclass
class ParsedValue:
    index: int
    start: int
    end: int
    value: Any
    structural_sha256: str

@dataclasses.dataclass
class ParseResult:
    declared: str
    observed: str
    status: str
    values: list[ParsedValue]
    issues: list[dict]
    recovery_mode: str


def issue(code: str, start: int | None = None, end: int | None = None, *, severity: str = 'error') -> dict:
    return {'code':code,'byte_start':start,'byte_end':end,'severity':severity}


def structural_tree(v: Any) -> Any:
    if isinstance(v, Number): return ['number', v.raw]
    if v is None: return ['null']
    if isinstance(v, bool): return ['boolean', v]
    if isinstance(v, str): return ['string', v]
    if isinstance(v, list): return ['array', [structural_tree(x) for x in v]]
    if isinstance(v, dict): return ['object', [[k,structural_tree(v[k])] for k in sorted(v)]]
    raise FoundationError('INTERNAL_JSON_TYPE')


def structural_digest(v: Any) -> str:
    return digest(b'recursiveintell:json-lexeme-tree:v1\0'+canonical_json(structural_tree(v)).encode())


def decoder(limits: Limits) -> json.JSONDecoder:
    def pairs(items):
        d = {}
        for k,v in items:
            if k in d: raise FoundationError('JSON_DUPLICATE_KEY')
            d[k]=v
        return d
    def number(s):
        if len(s)>limits.json_number_chars: raise FoundationError('JSON_NUMBER_LENGTH_LIMIT')
        return Number(s)
    def nonfinite(_): raise FoundationError('JSON_NONFINITE_NUMBER')
    return json.JSONDecoder(object_pairs_hook=pairs, parse_int=number, parse_float=number,
                            parse_constant=nonfinite, strict=True)


def guard_depth(text: str, limits: Limits) -> None:
    # Conservative lexical bounds BEFORE decoding, including flat arrays/maps.
    # Counts separators/string starts, not semantic nodes: rejection is explicit.
    depth=0; quoted=False; escape=False; tokens=0; string_chars=0
    for i,c in enumerate(text):
        if quoted:
            string_chars += 1
            if string_chars > limits.json_string_chars:
                raise FoundationError('JSON_STRING_LENGTH_LIMIT', offset=len(text[:i].encode()))
            if escape: escape=False
            elif c=='\\': escape=True
            elif c=='"': quoted=False
        elif c=='"':
            quoted=True; string_chars=0; tokens+=1
        elif c in '[{':
            depth+=1; tokens+=1
            if depth>limits.json_depth:
                raise FoundationError('JSON_DEPTH_LIMIT',offset=len(text[:i].encode()))
        elif c in ']}': depth-=1
        elif c in ',:': tokens+=1
        if tokens>limits.json_tokens:
            raise FoundationError('JSON_TOKEN_COUNT_LIMIT',offset=len(text[:i].encode()))


def guard_unicode(v: Any) -> None:
    todo=[v]
    while todo:
        x=todo.pop()
        if isinstance(x,dict): todo.extend(x.keys());todo.extend(x.values())
        elif isinstance(x,list):todo.extend(x)
        elif isinstance(x,str) and any(0xD800<=ord(c)<=0xDFFF for c in x):
            raise FoundationError('JSON_UNPAIRED_SURROGATE')


def strict_loads(raw: bytes, limits: Limits = Limits()) -> Any:
    result=parse(raw,'input.json',limits=limits,recover_streams=False)
    if result.status!='parsed' or len(result.values)!=1:
        raise FoundationError(result.issues[0]['code'] if result.issues else 'JSON_INVALID')
    return result.values[0].value


def plain(v: Any) -> Any:
    """Metadata conversion only, never applied to arbitrary historical payloads."""
    if isinstance(v,Number):
        if v.raw != '-0' and re.fullmatch(r'-?(0|[1-9][0-9]*)',v.raw):return int(v.raw)
        return {'numeric_lexeme':v.raw}
    if isinstance(v,list):return [plain(x) for x in v]
    if isinstance(v,dict):return {k:plain(x) for k,x in v.items()}
    return v


def parse(raw: bytes, path: str, *, limits: Limits = Limits(), recover_streams: bool = False,
          markdown_fences: bool = True) -> ParseResult:
    suffix=pathlib.PurePosixPath(path).suffix.lower()
    declared={'.json':'json','.jsonl':'jsonl','.ndjson':'ndjson','.md':'markdown','.markdown':'markdown'}.get(suffix,'unknown')
    mode='explicit-json-streams-v1' if recover_streams else 'strict-v1'
    if len(raw)>limits.member_bytes:
        return ParseResult(declared,'unknown','quarantined',[],[issue('PARSE_SIZE_LIMIT')],mode)
    try:text=raw.decode('utf-8','strict')
    except UnicodeDecodeError as e:
        return ParseResult(declared,'binary','unsupported',[],[issue('UTF8_INVALID',e.start,e.end)],mode)
    if raw.startswith(b'\xef\xbb\xbf'):
        return ParseResult(declared,'utf8-bom','quarantined',[],[issue('UTF8_BOM_NOT_ALLOWED',0,3)],mode)
    if '\x00' in text:
        return ParseResult(declared,'binary','unsupported',[],[issue('NUL_IN_TEXT')],mode)
    if declared=='unknown' and not text.lstrip().startswith(('{','[')):
        return ParseResult(declared,'text','classified_non_data',[],[],mode)
    if declared=='markdown':
        if not markdown_fences:
            return ParseResult(declared,'markdown','classified_non_data',[],[],mode)
        vals=[];issues=[]
        pattern=re.compile(r'^```(json|jsonl|ndjson)[ \t]*\r?\n(.*?)^```[ \t]*\r?$',re.M|re.S)
        for match in pattern.finditer(text):
            start=len(text[:match.start(2)].encode()); block=match.group(2).encode()
            part=parse(block,'fence.'+match.group(1),limits=limits,recover_streams=recover_streams)
            for val in part.values:
                vals.append(ParsedValue(len(vals),start+val.start,start+val.end,val.value,val.structural_sha256))
            for err in part.issues:
                err=dict(err)
                for k in ('byte_start','byte_end'):
                    if err[k] is not None:err[k]+=start
                issues.append(err)
        return ParseResult(declared,'markdown-fenced-json' if vals else 'markdown',
                           'parsed_with_issues' if vals and issues else 'parsed' if vals else 'quarantined' if issues else 'classified_non_data',vals,issues,mode)
    try:guard_depth(text,limits)
    except FoundationError as e:
        return ParseResult(declared,'json-like','quarantined',[],[issue(e.code,e.offset)],mode)
    dec=decoder(limits)
    ws=' \r\n\t'
    def single(s: str):
        start=len(s)-len(s.lstrip(ws))
        obj,end=dec.raw_decode(s,start);guard_unicode(obj)
        if s[end:].strip(ws):raise FoundationError('JSON_TRAILING_DATA',offset=len(s[:end].encode()))
        return obj,start,end
    def error(exc,base=0):
        if isinstance(exc,json.JSONDecodeError):
            return issue('JSON_SYNTAX',base+len(exc.doc[:exc.pos].encode()))
        if isinstance(exc,FoundationError):return issue(exc.code,base+exc.offset if exc.offset is not None else base)
        if isinstance(exc,RecursionError):return issue('JSON_DEPTH_LIMIT',base)
        raise exc
    if not text.strip(ws):
        return ParseResult(declared,'empty','quarantined',[],[issue('EMPTY_DOCUMENT',0,len(raw))],mode)
    values=[];issues=[]
    if declared in {'jsonl','ndjson'}:
        base=0;line_error=None;strict_values=[]
        for line in re.findall(r'[^\n]*\n|[^\n]+$', text):
            if len(strict_values)>=limits.json_values:
                line_error=issue('JSON_VALUE_COUNT_LIMIT',base);break
            if not line.strip(ws):line_error=issue('JSONL_BLANK_LINE',base,base+len(line.encode()));break
            try:
                obj,a,b=single(line)
                strict_values.append(ParsedValue(len(strict_values),base+len(line[:a].encode()),base+len(line[:b].encode()),obj,structural_digest(obj)))
            except (json.JSONDecodeError,FoundationError,RecursionError) as e:
                line_error=error(e,base);break
            base+=len(line.encode())
        if line_error is None:
            if declared=='ndjson' and not raw.endswith(b'\n'):
                issues.append(issue('NDJSON_MISSING_TERMINAL_LF',len(raw),len(raw),severity='warning'))
            return ParseResult(declared,'strict-'+declared,'parsed_with_issues' if issues else 'parsed',strict_values,issues,mode)
        if not recover_streams:
            return ParseResult(declared,'nonconformant-'+declared,'quarantined',[],[line_error,issue('STREAM_RECOVERY_NOT_ENABLED')],mode)
        issues.append(issue('DECLARED_LINE_FORMAT_NONCONFORMANT',line_error['byte_start'],severity='warning'))
    else:
        try:
            obj,a,b=single(text)
            values=[ParsedValue(0,len(text[:a].encode()),len(text[:b].encode()),obj,structural_digest(obj))]
            if declared=='unknown':issues.append(issue('UNDECLARED_JSON',severity='warning'))
            return ParseResult(declared,'json','parsed_with_issues' if issues else 'parsed',values,issues,mode)
        except (json.JSONDecodeError,FoundationError,RecursionError) as e:
            initial_error=error(e)
            if not recover_streams:return ParseResult(declared,'json-like','quarantined',[],[initial_error],mode)
            issues.append(issue('DECLARED_JSON_NONCONFORMANT',initial_error['byte_start'],severity='warning'))
    # Explicit historical recovery: sequential raw_decode; never scan past garbage.
    pos=0;bytepos=0;end_prev=0;pretty=False;concatenated=False
    while pos<len(text):
        start=pos
        while start<len(text) and text[start] in ws:start+=1
        bytepos+=len(text[pos:start].encode());pos=start
        if pos==len(text):break
        if len(values)>=limits.json_values:
            issues.append(issue('JSON_VALUE_COUNT_LIMIT',bytepos,len(raw)));break
        try:
            obj,end=dec.raw_decode(text,pos);guard_unicode(obj)
            chunk=text[pos:end].encode()
            if '\n' in text[pos:end]:pretty=True
            if values and pos==end_prev:concatenated=True
            values.append(ParsedValue(len(values),bytepos,bytepos+len(chunk),obj,structural_digest(obj)))
            bytepos+=len(chunk);pos=end;end_prev=end
        except (json.JSONDecodeError,FoundationError,RecursionError) as e:
            err=error(e)
            if err['byte_start'] is None or isinstance(e,FoundationError) and e.offset is None:err['byte_start']=bytepos
            err['byte_end']=len(raw);issues.append(err);break
    errors=any(x['severity']=='error' for x in issues)
    observed='pretty-json-stream' if pretty else 'concatenated-json-values' if concatenated else 'whitespace-json-stream'
    status='partial_quarantined' if values and errors else 'quarantined' if errors else 'parsed_with_issues'
    return ParseResult(declared,observed,status,values,issues,mode)
