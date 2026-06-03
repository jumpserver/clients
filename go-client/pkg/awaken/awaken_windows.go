package awaken

import (
	"crypto/rand"
	"encoding/json"
	"fmt"
	"go-client/global"
	"go-client/pkg/autoit"
	"go-client/pkg/config"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"time"
)

func EnsureDirExist(path string) {
	if fi, err := os.Stat(path); err == nil && fi.IsDir() {
		return
	}
	if err := os.MkdirAll(path, os.ModePerm); err != nil {
		global.LOG.Error(err.Error())
	}
}

func fileExists(filename string) bool {
	info, err := os.Stat(filename)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}

func getNavicatURL(connectInfo map[string]string) string {
	re := regexp.MustCompile(`@(.+)$`)
	matches := re.FindStringSubmatch(connectInfo["name"])
	name := connectInfo["username"]
	if len(matches) > 1 {
		name = matches[1]
	}
	url := fmt.Sprintf("navicat://conn.%s?Conn.Host=%s&Conn.Name=%s&Conn.Port=%s&Conn.Username=%s",
		connectInfo["protocol"], connectInfo["host"], name, connectInfo["port"], connectInfo["username"])
	switch connectInfo["protocol"] {
	case "oracle":
		url = strings.Replace(url, "conn.oracle", "conn.ora", 1)
		url += fmt.Sprintf("&Conn.ServiceName=%s&Conn.ServiceNameType=ServiceName&Conn.ConnectionMode=Basic", connectInfo["dbname"])
	case "sqlserver":
		url = strings.Replace(url, "conn.sqlserver", "conn.mssql", 1)
		url += fmt.Sprintf("&Conn.AuthenticationType=Default&Conn.InitialDatabase=%s", connectInfo["dbname"])
	case "postgresql":
		url = strings.Replace(url, "conn.postgresql", "conn.pgsql", 1)
		url += fmt.Sprintf("&Conn.InitialDatabase=%s", connectInfo["dbname"])
	}

	pattern := regexp.MustCompile(`[\^(){}~]`)
	url = pattern.ReplaceAllStringFunc(url, func(match string) string {
		return fmt.Sprintf("{%s}", match)
	})
	return url
}

func getCommandFromArgs(connectInfo map[string]string, argFormat string) string {
	for key, value := range connectInfo {
		argFormat = strings.Replace(argFormat, "{"+key+"}", value, 1)
	}
	return argFormat
}

// validateAppPath checks if the application path exists
func validateAppPath(appPath string) error {
	if appPath == "" {
		return fmt.Errorf("application path is empty")
	}
	// Check if path exists
	if _, err := os.Stat(appPath); os.IsNotExist(err) {
		return fmt.Errorf("application path does not exist: %s", appPath)
	}
	return nil
}

func handleRDP(r *Rouse, filePath string, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.Windows.RemoteDesktop
	for _, app := range appLst {
		if app.IsMatchProtocol("rdp") {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	appPath := appItem.Path
	if !appItem.IsInternal {
		if err := validateAppPath(appItem.Path); err != nil {
			global.LOG.Error(err.Error())
			return nil
		}
	}

	args := strings.Replace(appItem.ArgFormat, "{file}", filePath, 1)
	return exec.Command(appPath, args)
}

func handleVNC(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.Windows.RemoteDesktop
	for _, app := range appLst {
		if app.IsMatchProtocol("vnc") {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	if !appItem.IsInternal {
		if err := validateAppPath(appItem.Path); err != nil {
			global.LOG.Error(err.Error())
			return nil
		}
	}
	connectMap := map[string]string{
		"name":     r.getName(),
		"protocol": r.Protocol,
		"username": r.getUserName(),
		"value":    r.Value,
		"host":     r.Host,
		"port":     strconv.Itoa(r.Port),
	}
	if appItem.Name == "realvnc" {
		dir, _ := os.UserConfigDir()
		currentPath := filepath.Join(dir, "jumpserver-client")
		EnsureDirExist(currentPath)
		files, _ := ioutil.ReadDir(currentPath)
		for _, file := range files {
			if !file.IsDir() && strings.EqualFold(filepath.Ext(file.Name()), ".vnc") {
				os.Remove(filepath.Join(currentPath, file.Name()))
			}
		}
		filePath := filepath.Join(currentPath, r.getName()+".vnc")
		content := fmt.Sprintf("[Connection]\nHost=%s:%s\nWarnUnencrypted=0\nUserName=%s\n", r.Host, strconv.Itoa(r.Port), r.getUserName())
		if err := ioutil.WriteFile(filePath, []byte(content), os.ModePerm); err != nil {
			global.LOG.Error(err.Error())
			return nil
		}
		autoit.LoadAuto()
		autoit.Run(appItem.Path + " -VerifyId \"0\" \"" + filePath + "\"")
		winTitle := "Authentication"
		active := false
		for i := 0; i <= 30; i++ {
			ret := autoit.WinWaitActive(winTitle, "", 1)
			time.Sleep(300 * time.Millisecond)
			if ret != 0 {
				active = true
				break
			}
			autoit.WinActivate(winTitle, "")
		}
		if !active {
			autoit.WinActivate(winTitle, "")
			time.Sleep(500 * time.Millisecond)
		}
		focusedControl := autoit.ControlGetFocus(winTitle, "")
		focusedText := strings.TrimSpace(autoit.ControlGetText(winTitle, "", focusedControl))
		if focusedControl != "" && focusedText == strings.TrimSpace(r.getUserName()) {
			autoit.Send("{TAB}")
			time.Sleep(300 * time.Millisecond)
		}
		autoit.Send("^a")
		time.Sleep(100 * time.Millisecond)
		autoit.Send(r.Value, 1)
		autoit.Send("{ENTER}")
		return exec.Command("cmd", "/C", "exit", "0")
	}
	if len(appItem.AutoIt) == 0 {
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		cmd := exec.Command(appItem.Path, strings.Split(commands, " ")...)
		// 设置环境变量（只对这个子进程有效）
		cmd.Env = append(os.Environ(),
			"VNC_USERNAME="+r.getUserName(),
			"VNC_PASSWORD="+r.Value,
		)
		return cmd
	} else {
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		global.LOG.Error(appItem.Path + " " + commands)
		autoit.LoadAuto()
		autoit.Run(appItem.Path + " " + commands)
		for _, item := range appItem.AutoIt {
			time.Sleep(300 * time.Millisecond)
			switch item.Cmd {
			case "Wait":
				sleepTime, _ := strconv.Atoi(item.Type)
				winTitle := item.Element
				maxRetry := 0
				for {
					ret := autoit.WinWaitActive(winTitle, "", sleepTime)
					time.Sleep(time.Duration(sleepTime) * 100 * time.Millisecond)
					if ret != 0 || maxRetry > 30 {
						break
					}
					maxRetry++
				}
			case "ControlSend":
				maxRetry := 0
				for {
					ret := autoit.ControlSend("", "", item.Element, getCommandFromArgs(connectMap, item.Type))
					time.Sleep(300 * time.Millisecond)
					if ret != 0 || maxRetry > 10 {
						break
					}
					maxRetry++
				}
			case "ControlSetText":
				maxRetry := 0
				for {
					ret := autoit.ControlSetText("", "", item.Element, getCommandFromArgs(connectMap, item.Type))
					time.Sleep(300 * time.Millisecond)
					if ret != 0 || maxRetry > 10 {
						break
					}
					maxRetry++
				}
			case "ControlClick":
				pos := strings.Split(item.Type, ",")
				x, _ := strconv.Atoi(pos[0])
				y, _ := strconv.Atoi(pos[1])
				maxRetry := 0
				for {
					ret := autoit.ControlClick("", "", item.Element, "left", 1, x, y)
					time.Sleep(300 * time.Millisecond)
					if ret != 0 || maxRetry > 10 {
						break
					}
					maxRetry++
				}
			case "SendKey":
				autoit.Send(item.Element)
			}
		}
		return exec.Command("")
	}
}

func handleSSH(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	var appLst []config.AppItem
	switch r.Protocol {
	case "ssh", "telnet":
		appLst = cfg.Windows.Terminal
	case "sftp":
		appLst = cfg.Windows.FileTransfer
	}

	for _, app := range appLst {
		if app.IsMatchProtocol(r.Protocol) {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	// telnet 协议使用 ssh 的配置参数格式
	protocol := r.Protocol
	if protocol == "telnet" {
		protocol = "ssh"
	}

	var appPath string
	if appItem.IsInternal {
		currentPath, _ := filepath.Abs(filepath.Dir(os.Args[0]))
		appPath = filepath.Join(currentPath, appItem.Path)
	} else {
		appPath = appItem.Path
	}
	if !appItem.IsInternal {
		if err := validateAppPath(appItem.Path); err != nil {
			global.LOG.Error(err.Error())
			return nil
		}
	}
	connectMap := map[string]string{
		"name":     r.getName(),
		"protocol": protocol,
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

// prepareWorkbenchConnection writes a minimal connection entry into MySQL Workbench's
// connections.xml so that the connection succeeds without OS mismatch warnings.
func prepareWorkbenchConnection(connName, host, port, username string) error {
	dir, err := os.UserConfigDir()
	if err != nil {
		return fmt.Errorf("failed to get user config dir: %w", err)
	}
	wbDir := filepath.Join(dir, "MySQL", "Workbench")
	EnsureDirExist(wbDir)
	connFile := filepath.Join(wbDir, "connections.xml")

	// Generate a UUID for the connection entry
	b := make([]byte, 16)
	rand.Read(b)
	connID := fmt.Sprintf("{%08X-%04X-%04X-%04X-%012X}",
		b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])

	// Build the minimal connection XML entry
	connEntry := fmt.Sprintf(`
    <value type="object" struct-name="db.mgmt.Connection" id="%s" struct-checksum="0x96ba47d8">
      <link type="object" struct-name="db.mgmt.Driver" key="driver">com.mysql.rdbms.mysql.driver.native</link>
      <value type="string" key="hostIdentifier">Mysql@%s:%s</value>
      <value type="int" key="isDefault">0</value>
      <value type="dict" key="modules"/>
      <value type="dict" key="parameterValues">
        <value type="string" key="hostName">%s</value>
        <value type="int" key="port">%s</value>
        <value type="string" key="userName">%s</value>
      </value>
      <value type="string" key="name">%s</value>
    </value>`,
		connID, host, port, host, port, username, connName)

	var data []byte
	if !fileExists(connFile) {
		content := fmt.Sprintf(`<?xml version="1.0"?>
<data grt_format="2.0">
  <value type="list" content-type="object" content-struct-name="db.mgmt.Connection">%s
  </value>
</data>`, connEntry)
		data = []byte(content)
	} else {
		existing, err := ioutil.ReadFile(connFile)
		if err != nil {
			return fmt.Errorf("failed to read connections.xml: %w", err)
		}
		content := string(existing)

		// Remove existing JumpServer connection if it exists
		re := regexp.MustCompile(`(?s)<value type="object" struct-name="db\.mgmt\.Connection"[^>]*>.*?<value type="string" key="name">JumpServer</value>.*?</value>`)
		content = re.ReplaceAllString(content, "")

		insertPos := strings.Index(content, "</value>\n</data>")
		if insertPos == -1 {
			insertPos = strings.Index(content, "</value>\r\n</data>")
		}
		if insertPos != -1 {
			content = content[:insertPos] + connEntry + "\n  " + content[insertPos:]
			data = []byte(content)
		} else {
			content := fmt.Sprintf(`<?xml version="1.0"?>
<data grt_format="2.0">
  <value type="list" content-type="object" content-struct-name="db.mgmt.Connection">%s
  </value>
</data>`, connEntry)
			data = []byte(content)
		}
	}

	err = ioutil.WriteFile(connFile, data, 0644)
	if err != nil {
		return fmt.Errorf("failed to write connections.xml: %w", err)
	}

	return nil
}

func handleDB(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	var appItem *config.AppItem
	appLst := cfg.Windows.Databases
	for _, app := range appLst {
		if app.IsMatchProtocol(r.Protocol) {
			appItem = &app
			break
		}
	}
	if appItem == nil {
		return nil
	}
	appPath := appItem.Path
	if !appItem.IsInternal {
		if err := validateAppPath(appItem.Path); err != nil {
			global.LOG.Error(err.Error())
			return nil
		}
	}

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
	if r.Protocol == "sqlserver" && appItem.Name == "dbeaver" {
		connectMap["protocol"] = "mssql_jdbc_ms_new"
	}
	if appItem.Name == "heidisql" {
		switch r.Protocol {
		case "mysql", "mariadb":
			connectMap["nettype"] = "0"
			connectMap["library"] = "libmariadb.dll"
		case "postgresql":
			connectMap["nettype"] = "8"
			connectMap["library"] = "libpq.dll"
		case "sqlserver":
			connectMap["nettype"] = "4"
			connectMap["library"] = "SQLOLEDB"
		}
	}
	if r.Protocol == "redis" && appItem.Name == "resp" {
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
		dir, _ := os.UserConfigDir()
		currentPath := filepath.Join(dir, "jumpserver-client")
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
	if appItem.Name == "navicat17" {
		url := getNavicatURL(connectMap)
		connectMap["url"] = url
	}
	if appItem.Name == "mysqlworkbench" {
		connName := "JumpServer"
		err := prepareWorkbenchConnection(connName, r.Host, strconv.Itoa(r.Port), r.getUserName())
		if err != nil {
			global.LOG.Error("Failed to prepare MySQL Workbench connection: " + err.Error())
			return nil
		}
		global.LOG.Info("MySQL Workbench connection prepared: " + connName)

		autoit.LoadAuto()
		
		// Run Workbench with --query <connection name> to directly open the connection tab
		global.LOG.Info("Launching MySQL Workbench with query: " + connName)
		autoit.Run(fmt.Sprintf(`"%s" --query "%s"`, appPath, connName))

		// Wait for password prompt dialog
		// The title is "Connect to MySQL Server"
		pwdTitle := "[REGEXPTITLE:.*Connect to MySQL Server.*]"
		active := false
		for i := 0; i <= 30; i++ {
			ret := autoit.WinWaitActive(pwdTitle, "", 1)
			time.Sleep(300 * time.Millisecond)
			if ret != 0 {
				active = true
				break
			}
			autoit.WinActivate(pwdTitle, "")
		}
		if active {
			time.Sleep(500 * time.Millisecond)
			// Send password in raw mode and press Enter
			autoit.Send(r.Value, 1)
			time.Sleep(100 * time.Millisecond)
			autoit.Send("{ENTER}")
		} else {
			global.LOG.Error("MySQL Workbench password dialog not detected")
		}
		return exec.Command("cmd", "/C", "exit", "0")
	}
	if len(appItem.AutoIt) == 0 {
		commands := getCommandFromArgs(connectMap, appItem.ArgFormat)
		if appItem.Name == "heidisql" && r.Protocol == "postgresql" {
			commands += " -db=" + r.DBName
		}
		if strings.Contains(commands, "*") {
			commands := strings.Split(commands, "*")
			return exec.Command(appPath, commands...)
		} else {
			commands := strings.Split(commands, " ")
			return exec.Command(appPath, commands...)
		}
	} else {
		autoit.LoadAuto()
		autoit.Run(appPath)
		for _, item := range appItem.AutoIt {
			time.Sleep(300 * time.Millisecond)
			switch item.Cmd {
			case "Wait":
				sleepTime, _ := strconv.Atoi(item.Type)
				winTitle := item.Element
				maxRetry := 0
				for {
					ret := autoit.WinWaitActive(winTitle, "", sleepTime)
					time.Sleep(time.Duration(sleepTime) * 100 * time.Millisecond)
					if ret != 0 || maxRetry > 30 {
						break
					}
					maxRetry++
				}
			case "ControlSend":
				maxRetry := 0
				for {
					ret := autoit.ControlSend("", "", item.Element, getCommandFromArgs(connectMap, item.Type))
					time.Sleep(300 * time.Millisecond)
					if ret != 0 || maxRetry > 10 {
						break
					}
					maxRetry++
				}
			case "ControlSetText":
				maxRetry := 0
				for {
					ret := autoit.ControlSetText("", "", item.Element, getCommandFromArgs(connectMap, item.Type))
					time.Sleep(300 * time.Millisecond)
					if ret != 0 || maxRetry > 10 {
						break
					}
					maxRetry++
				}
			case "ControlClick":
				pos := strings.Split(item.Type, ",")
				x, _ := strconv.Atoi(pos[0])
				y, _ := strconv.Atoi(pos[1])
				maxRetry := 0
				for {
					ret := autoit.ControlClick("", "", item.Element, "left", 1, x, y)
					time.Sleep(300 * time.Millisecond)
					if ret != 0 || maxRetry > 10 {
						break
					}
					maxRetry++
				}
			case "SendKey":
				autoit.Send(item.Element)
			}
		}
		return exec.Command("")
	}
}

func handleCommand(r *Rouse, cfg *config.AppConfig) *exec.Cmd {
	cmd := exec.Command(r.Command)
	return cmd
}
