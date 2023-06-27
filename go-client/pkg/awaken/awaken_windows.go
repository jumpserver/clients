package awaken

import (
	"go-client/pkg/config"
	"os/exec"
	"path/filepath"
	"strings"
)

func getCommandFromArgs(connectInfo map[string]string, argFormat string) string {
	for key, value := range connectInfo {
		argFormat = strings.Replace(argFormat, "{"+key+"}", value, 1)
	}
	return argFormat
}

func handleRDP(r *Rouse, filePath string, cfg *config.AppConfig) *exec.Cmd {
	cmd := exec.Command("mstsc.exe", filePath)
	return cmd
}

func handleSSH(r *Rouse, currentPath string, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.Windows.Terminal
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
		appPath = filepath.Join(currentPath, appItem.Path)
	} else {
		appPath = appItem.Path
	}

	connectMap := map[string]string{
		"name":     r.Name,
		"protocol": r.Protocol,
		"username": r.Username,
		"value":    r.Value,
		"host":     r.Host,
		"port":     r.Port,
	}
	commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
	return exec.Command(appPath, strings.Split(commands, " ")...)
}

func structureMySQLCommand(command string) string {
	return ""
}

func structureRedisCommand(command string) string {
	return ""
}

func structurePostgreSQLCommand(command string) string {
	return ""
}

func handleDB(r *Rouse, command string, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.Windows.Databases
	for _, app := range appLst {
		if app.IsActive() && app.IsSupportProtocol(r.Protocol) {
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
		"port":     r.Port,
		"dbname":   r.DBName,
	}
	commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
	return exec.Command(appPath, strings.Split(commands, " ")...)
}
