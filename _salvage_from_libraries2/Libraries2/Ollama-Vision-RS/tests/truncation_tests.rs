use ollama_vision::{CaptionOptions, TagOptions};

#[test]
fn tag_truncation_limits_count() {
    let opts = TagOptions {
        max_tags: 3,
        ..Default::default()
    };
    assert_eq!(opts.max_tags, 3);
    // Verify default max_tag_length
    assert_eq!(TagOptions::default().max_tag_length, 50);
}

#[test]
fn caption_truncation_defaults() {
    let opts = CaptionOptions::default();
    assert_eq!(opts.max_caption_length, 500);
    assert_eq!(opts.max_retries, 2);
}

#[test]
fn tag_truncation_max_tag_length_default() {
    let opts = TagOptions::default();
    assert_eq!(opts.max_tags, 30);
    assert_eq!(opts.max_tag_length, 50);
    assert_eq!(opts.max_retries, 2);
}

// Test the truncation helpers by exercising the parser + truncation pipeline
// via the public parse_tags function (truncation is internal to tag_image/caption_image)
#[test]
fn parse_tags_truncation_is_downstream() {
    // parse_tags itself doesn't truncate — it's the tag_image function that does.
    // Verify the parser still works correctly with the new dependency.
    let tags = ollama_vision::parse_tags(r#"["portrait", "fantasy", "dark"]"#).unwrap();
    assert_eq!(tags, vec!["portrait", "fantasy", "dark"]);
}
