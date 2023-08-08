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
	cmd := exec.Command("remmina", filePath)
	return cmd
}

func awakenSSHCommand(r *Rouse, currentPath string, cfg *config.AppConfig) *exec.Cmd {
	clientPath := filepath.Join(currentPath, "client")
	command := fmt.Sprintf("%s %s -P %s", clientPath, r.Command, r.Value)
	cmd := new(exec.Cmd)
	out, _ := exec.Command("bash", "-c", "echo $XDG_CURRENT_DESKTOP").CombinedOutput()
	currentDesktop := strings.ToLower(strings.Trim(string(out), "\n"))

	switch currentDesktop {
	case "gnome":
		cmd = exec.Command(
			"gnome-terminal", "--", "bash", "-c",
			fmt.Sprintf("%s; exec bash -i", command),
		)
	case "deepin":
		cmd = exec.Command("deepin-terminal", "--keep-open", "-C", command)
	default:
		msg := fmt.Sprintf("Not yet supported %s desktop system", currentDesktop)
		global.LOG.Info(msg)
	}
	return cmd
}

func awakenDBCommand(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
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
