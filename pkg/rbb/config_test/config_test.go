package config_test

import (
	config2 "otel-arrow-adapter/pkg/rbb/config"
	"testing"
)

func TestIsDictionary(t *testing.T) {
	t.Parallel()
	config := config2.DictionaryConfig{
		MinRowCount:           10,
		MaxCard:               2,
		MaxCardRatio:          0.5,
		MaxSortedDictionaries: 5,
	}

	if !config.IsDictionary(10, 1) {
		t.Errorf("Expected a dictionary")
	}
	if !config.IsDictionary(10, 2) {
		t.Errorf("Expected a dictionary")
	}

	if config.IsDictionary(5, 1) {
		t.Errorf("Didn't expect a dictionary (too few rows)")
	}
	if config.IsDictionary(10, 3) {
		t.Errorf("Didn't rxpect a dictionary (too many unique values")
	}
}
