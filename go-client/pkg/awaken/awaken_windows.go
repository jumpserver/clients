package awaken

import (
	"encoding/json"
	"go-client/global"
	"go-client/pkg/autoit"
	"go-client/pkg/config"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

func EnsureDirExist(path string) {
	if fi, err := os.Stat(path); err == nil && fi.IsDir() {
		return
	}
	if err := os.MkdirAll(path, os.ModePerm); err != nil {
		global.LOG.Error(err.Error())
	}
}

func getCommandFromArgs(connectInfo map[string]string, argFormat string) string {
	for key, value := range connectInfo {
		argFormat = strings.Replace(argFormat, "{"+key+"}", value, 1)
	}
	return argFormat
}

func handleRDP(r *Rouse, filePath string, cfg *config.AppConfig) *exec.Cmd {
	cmd := exec.Command("C:\\WINDOWS\\system32\\mstsc.exe", filePath)
	return cmd
}

func handleSSH(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	var appLst []config.AppItem
	switch r.Protocol {
	case "ssh", "telnet":
		r.Protocol = "ssh"
		appLst = cfg.Windows.Terminal
	case "sftp":
		appLst = cfg.Windows.FileTransfer
	}

	for _, app := range appLst {
		if app.IsActive() && app.IsSupportProtocol(r.Protocol) {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	var appPath string
	if appItem.IsInternal {
		currentPath := filepath.Dir(os.Args[0])
		appPath = filepath.Join(currentPath, appItem.Path)
	} else {
		appPath = appItem.Path
	}

	connectMap := map[string]string{
		"name":     r.getName(),
		"protocol": r.Protocol,
		"username": r.getUserName(),
		"value":    r.Value,
		"host":     r.Host,
		"port":     strconv.Itoa(r.Port),
	}
	commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
	if strings.Contains(commands, "*") {
		commands := strings.Split(commands, "*")
		return exec.Command(appPath, commands[0], commands[1])
	} else {
		commands := strings.Split(commands, " ")
		return exec.Command(appPath, commands...)
	}
}

func handleDB(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.Windows.Databases
	for _, app := range appLst {
		if app.IsSet && app.IsMatchProtocol(r.Protocol) {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	appPath := appItem.Path

	connectMap := map[string]string{
		"name":     r.getName(),
		"protocol": r.Protocol,
		"username": r.getUserName(),
		"value":    r.Value,
		"host":     r.Host,
		"port":     strconv.Itoa(r.Port),
		"dbname":   r.DBName,
	}

	if r.Protocol == "sqlserver" {
		connectMap["protocol"] = "mssql_jdbc_ms_new"
	}
	if r.Protocol == "redis" {
		if appItem.Name == "resp" {
			var conList []map[string]string
			ss := make(map[string]string)
			ss["host"] = r.Host
			ss["port"] = strconv.Itoa(r.Port)
			ss["name"] = r.getName()
			ss["auth"] = r.Token.ID + "@" + r.Value
			ss["ssh_agent_path"] = ""
			ss["ssh_password"] = ""
			ss["ssh_private_key_path"] = ""
			ss["timeout_connect"] = "60000"
			ss["timeout_execute"] = "60000"
			conList = append(conList, ss)

			bjson, _ := json.Marshal(conList)
			currentPath := filepath.Dir(os.Args[0])
			rdmPath := filepath.Join(currentPath, ".rdm")
			EnsureDirExist(rdmPath)
			filePath := filepath.Join(rdmPath, "connections.json")
			global.LOG.Error(filePath)
			err := ioutil.WriteFile(filePath, bjson, os.ModePerm)
			if err != nil {
				global.LOG.Error(err.Error())
				return nil
			}
			connectMap["config_file"] = currentPath
		}
	}
	if len(appItem.AutoIt) == 0 {
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		if strings.Contains(commands, "*") {
			commands := strings.Split(commands, "*")
			return exec.Command(appPath, commands...)
		} else {
			commands := strings.Split(commands, " ")
			return exec.Command(appPath, commands...)
		}
	} else {
		autoit.Run(appPath)
		winTitle := ""
		for _, item := range appItem.AutoIt {
			switch item.Cmd {
			case "Wait":
				winTitle = item.Element
				autoit.WinWait(winTitle, "", 120)
				autoit.WinActivate(winTitle)
			case "ControlSend":
				autoit.ControlSend(winTitle, "", item.Element, getCommandFromArgs(connectMap, item.Type))
			case "ControlClick":
				autoit.ControlClick(winTitle, "", item.Element)
			}
		}
		return exec.Command("")
	}
}

func handleCommand(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	cmd := exec.Command(r.Command)
	return cmd
}
