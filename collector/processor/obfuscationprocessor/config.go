package obfuscationprocessor

type Config struct {
	// Rounds is a Fiestel parameter which determines the
	// difficulty of uncovering the original data.  Default 10.
	Rounds int `mapstructure:"rounds"`

	// KeyLength is a Fiestel parameter which determines the
	// length of the keyt used to obfuscate.  Default 128.
	KeyLength int `mapstructure:"key_length"`

	// EncryptAll indicates that all byte-array and string values
	// should be obfuscated.
	EncryptAll bool `mapstructure:"encrypt_all"`

	// EncryptAttributes indicates a specific list of attributes
	// to obfuscate.
	EncryptAttributes []string `mapstructure:"encrypt_attributes"`
}
