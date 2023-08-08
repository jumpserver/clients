package config

import (
	"encoding/json"
	"go-client/global"
	"os"
	"path/filepath"
)

type AppConfig struct {
	FileName string  `json:"filename"`
	Windows  AppType `json:"windows"`
	MacOS    AppType `json:"macos"`
	Linux    AppType `json:"linux"`
}

type AppType struct {
	Terminal      []AppItem `json:"terminal"`
	FileTransfer  []AppItem `json:"filetransfer"`
	RemoteDesktop []AppItem `json:"remotedesktop"`
	Databases     []AppItem `json:"databases"`
}

type AppItem struct {
	Name        string   `json:"name"`
	DisplayName string   `json:"display_name"`
	Protocol    []string `json:"protocol"`
	Comment     string   `json:"comment"`
	Type        string   `json:"type"`
	MatchFirst  []string `json:"match_first"`
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

func (a *AppItem) IsSupportProtocol(protocol string) bool {
	for _, p := range a.Protocol {
		if p == protocol {
			return true
		}
	}
	return false
}

func (a *AppItem) IsMatchProtocol(protocol string) bool {
	for _, p := range a.MatchFirst {
		if p == protocol {
			return true
		}
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
	//filePath := filepath.Join(filepath.Dir(os.Args[0]), "config.json")
	filePath := filepath.Join("/Users/halo/golang/clients/interface/bin/config.json")
	jsonFile, err := os.Open(filePath)
	if err != nil {
		global.LOG.Error(err.Error())
	}
	defer func(jsonFile *os.File) {
		err := jsonFile.Close()
		if err != nil {
			global.LOG.Error(err.Error())
		}
	}(jsonFile)
	decoder := json.NewDecoder(jsonFile)
	err = decoder.Decode(&GlobalConfig)
	if err != nil {
		global.LOG.Error(err.Error())
		return AppConfig{}
	}
	return *GlobalConfig
}
