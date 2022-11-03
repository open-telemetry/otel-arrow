package internal

import "go.opentelemetry.io/collector/pdata/pcommon"

func Attrs1() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutStr("str", "string1")
	attrs.PutInt("int", 1)
	attrs.PutDouble("double", 1.0)
	attrs.PutBool("bool", true)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes1")...)
	return attrs
}

func Attrs2() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutStr("str", "string2")
	attrs.PutInt("int", 2)
	attrs.PutDouble("double", 2.0)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes2")...)
	return attrs
}

func Attrs3() pcommon.Map {
	attrs := pcommon.NewMap()
	attrs.PutStr("str", "string3")
	attrs.PutDouble("double", 3.0)
	attrs.PutBool("bool", false)
	bytes := attrs.PutEmptyBytes("bytes")
	bytes.Append([]byte("bytes3")...)
	return attrs
}

func Scope1() pcommon.InstrumentationScope {
	scope := pcommon.NewInstrumentationScope()
	scope.SetName("scope1")
	scope.SetVersion("1.0.1")
	scopeAttrs := scope.Attributes()
	Attrs1().CopyTo(scopeAttrs)
	scope.SetDroppedAttributesCount(0)
	return scope
}

func Scope2() pcommon.InstrumentationScope {
	scope := pcommon.NewInstrumentationScope()
	scope.SetName("scope2")
	scope.SetVersion("1.0.2")
	scopeAttrs := scope.Attributes()
	Attrs2().CopyTo(scopeAttrs)
	scope.SetDroppedAttributesCount(1)
	return scope
}

func Resource1() pcommon.Resource {
	resource := pcommon.NewResource()
	resourceAttrs := resource.Attributes()
	Attrs1().CopyTo(resourceAttrs)
	resource.SetDroppedAttributesCount(0)
	return resource
}

func Resource2() pcommon.Resource {
	resource := pcommon.NewResource()
	resourceAttrs := resource.Attributes()
	Attrs2().CopyTo(resourceAttrs)
	resource.SetDroppedAttributesCount(1)
	return resource
}
