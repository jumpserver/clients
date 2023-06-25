package config

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
)

type AppConfig struct {
	FileName string    `json:"filename"`
	windows  []AppItem `json:"app_items"`
	macos  []AppItem `json:"app_items"`
	linux  []AppItem `json:"app_items"`

}

type AppItem struct {
	Name        string   `json:"name"`
	DisplayName string   `json:"display_name"`
	Protocol    []string `json:"protocol"`
	Comment     string   `json:"comment"`
	Type        string   `json:"type"`
	Path        string   `json:"path"`
	ArgFormat   string   `json:"arg_format"`
	IsInternal  bool     `json:"is_internal"`
	IsDefault   bool     `json:"is_default"`
	IsSet       bool     `json:"is_set"`
}

func (a *AppItem) IsActive() bool {
	if a.IsDefault && a.IsSet {
		return true
	}
	return false
}

func GetConf() AppConfig {
	if GlobalConfig == nil {
		return getDefaultConfig()
	}
	return *GlobalConfig
}

var GlobalConfig *AppConfig

func getDefaultConfig() AppConfig {
	filePath := filepath.Join("config.json")
	jsonFile, err := os.Open(filePath)
	if err != nil {
		log.Fatalln("Cannot open config file", err)
	}
	defer jsonFile.Close()
	decoder := json.NewDecoder(jsonFile)
	err = decoder.Decode(&GlobalConfig)
	if err != nil {
		fmt.Println("Cannot get configuration from file")
		return AppConfig{}
	}
	return *GlobalConfig
}
