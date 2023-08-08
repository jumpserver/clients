package awaken

import (
	"fmt"
	"go-client/global"
	"go-client/pkg/config"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

func getCommandFromArgs(connectInfo map[string]string, argFormat string) string {
	for key, value := range connectInfo {
		argFormat = strings.Replace(argFormat, "{"+key+"}", value, 1)
	}
	return argFormat
}

func awakenRDPCommand(filePath string) *exec.Cmd {
	global.LOG.Debug(filePath)
	cmd := exec.Command("open", filePath)
	return cmd
}

func awakenSSHCommand(r *Rouse, currentPath string, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	var appLst []config.AppItem
	switch r.Protocol {
	case "ssh":
		appLst = cfg.MacOS.Terminal
	case "sftp":
		appLst = cfg.MacOS.FileTransfer
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
	var cmd *exec.Cmd
	if appItem.IsInternal {
		clientPath := filepath.Join(currentPath, "client")
		command := fmt.Sprintf("%s %s -P %s", clientPath, r.Command, r.Value)
		cmd = exec.Command(
			"osascript", "-s", "h",
			"-e", fmt.Sprintf(`tell application "%s" to do script "%s"`, appItem.DisplayName, command),
		)
	} else {
		var appPath string
		appPath = appItem.Path

		connectMap := map[string]string{
			"name":     r.Name,
			"protocol": r.Protocol,
			"username": r.Username,
			"value":    r.Value,
			"host":     r.Host,
			"port":     strconv.Itoa(r.Port),
		}
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		appPath = appItem.Path
		cmd = exec.Command(appPath, strings.Split(commands, " ")...)
	}
	return cmd
}

func awakenDBCommand(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.MacOS.Databases
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
		"name":     r.Name,
		"protocol": r.Protocol,
		"username": r.Username,
		"value":    r.Value,
		"host":     r.Host,
		"port":     strconv.Itoa(r.Port),
		"dbname":   r.DBName,
	}
	commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
	return exec.Command(appPath, strings.Split(commands, " ")...)
}
