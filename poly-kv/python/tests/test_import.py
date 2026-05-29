def test_import_package_without_requiring_native_extension():
    import poly_kv

    assert hasattr(poly_kv, "ShapeV2")
    assert isinstance(poly_kv.native_available(), bool)
