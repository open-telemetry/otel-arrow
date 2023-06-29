package obfuscationprocessor

type Config struct {
	EncryptKey   string `mapstructure:"encrypt_key"`
	EncryptRound int    `mapstructure:"encrypt_round"`
	EncryptAll   bool   `mapstructure:"encrypt_all"`

	EncryptAttributes []string `mapstructure:"encrypt_attributes"`
}