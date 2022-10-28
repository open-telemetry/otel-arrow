package traces

import "github.com/f5/otel-arrow-adapter/pkg/otel/constants"

type Constants struct{}

func (_ Constants) ResourceEntitiesLabel() string {
	return constants.RESOURCE_SPANS
}
func (_ Constants) ScopeEntitiesLabel() string {
	return constants.SCOPE_SPANS
}
func (_ Constants) EntitiesLabel() string {
	return constants.SPANS
}
