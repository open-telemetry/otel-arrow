package memorylimiterextension // import "go.opentelemetry.io/collector/extension/memorylimiterextension"

type Config struct {
	MemoryLimitMiB uint32 `mapstructure:"memory_limit_mib"`
}
