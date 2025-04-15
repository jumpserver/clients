package awaken

import (
	"fmt"
	"go-client/global"
	"go-client/pkg/config"
	"os"
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

func awakenRDPCommand(filePath string, cfg *config.AppConfig) *exec.Cmd {
	global.LOG.Debug(filePath)
	cmd := exec.Command("open", filePath)
	return cmd
}

func awakenSSHCommand(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	var appLst []config.AppItem
	switch r.Protocol {
	case "ssh", "telnet":
		r.Protocol = "ssh"
		appLst = cfg.MacOS.Terminal
	case "sftp":
		appLst = cfg.MacOS.FileTransfer
	}

	for _, app := range appLst {
		if app.IsSet && app.IsMatchProtocol(r.Protocol) {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	var cmd *exec.Cmd
	connectMap := map[string]string{
		"name":     r.getName(),
		"protocol": r.Protocol,
		"username": r.getUserName(),
		"value":    r.Value,
		"host":     r.Host,
		"port":     strconv.Itoa(r.Port),
	}

	if appItem.IsInternal {
		currentPath := filepath.Dir(os.Args[0])
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		clientPath := filepath.Join(currentPath, "client")
		if appItem.Name == "iterm" {
			itermCmd := fmt.Sprintf(`%s %s`, clientPath, commands)

			scriptPath := filepath.Join(currentPath, "Scripts", "iterm2_loader.scpt")
			//command = fmt.Sprintf(`%s "%s"`, scriptPath, command)
			cmd = exec.Command("osascript", "-s", "h", scriptPath, itermCmd, "0")
		} else {
			cmd = exec.Command(
				"osascript", "-s", "h", "-e", fmt.Sprintf(`tell application "%s" to do script "%s %s" activate`,
					appItem.DisplayName, clientPath, commands),
			)
		}

	} else {
		var appPath string
		appPath = appItem.Path
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
		"name":     r.getName(),
		"protocol": r.Protocol,
		"username": r.getUserName(),
		"value":    r.Value,
		"host":     r.Host,
		"port":     strconv.Itoa(r.Port),
		"dbname":   r.DBName,
	}
	if r.Protocol == "oracle" {
		connectMap["dbname"] = r.getUserName()
	}
	if appItem.IsInternal {
		var argFormat string
		switch r.Protocol {
		case "redis":
			argFormat = "redis-cli -h {host} -p {port} -a {username}@{value}"
		case "oracle":
			argFormat = "sqlplus {username}/{value}@{host}:{port}/{dbname}"
		case "postgresql":
			argFormat = "psql 'user={username} password={value} host={host} dbname={dbname} port={port}'"
		case "mysql", "mariadb":
			argFormat = "mysql -u {username} -p{value} -h {host} -P {port} {dbname}"
		case "sqlserver":
			argFormat = "sqlcmd -S {host},{port} -U {username} -P {value} -d {dbname}"
		}
		commands := getCommandFromArgs(connectMap, argFormat)
		cmd := exec.Command(
			"osascript", "-s", "h", "-e", fmt.Sprintf(`tell application "%s" to do script "%s" activate`,
				appItem.DisplayName, commands),
		)
		return cmd
	} else {
		if r.Protocol == "sqlserver" {
			connectMap["protocol"] = "mssql_jdbc_ms_new"
		}
		appPath = appItem.Path
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		return exec.Command(appPath, strings.Split(commands, " ")...)
	}
}

func awakenOtherCommand(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var command string
	if r.Protocol == "ssh" {
		currentPath := filepath.Dir(os.Args[0])
		clientPath := filepath.Join(currentPath, "client")
		command = fmt.Sprintf("%s %s -P %s", clientPath, r.Command, r.Value)
	} else {
		command = r.Command
	}
	cmd := exec.Command(
		"osascript", "-s", "h",
		"-e", fmt.Sprintf(`tell application "%s" to do script "%s"`, "Terminal", command),
	)
	return cmd
}
