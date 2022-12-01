package awaken

import (
	"fmt"
	"go-client/global"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"
)

func handleRDP(filePath string) *exec.Cmd {
	cmd := exec.Command("mstsc.exe", filePath)
	return cmd
}

func handleSSH(c string, secret string, currentPath string) *exec.Cmd {
	command := ""
	if strings.HasPrefix(c, "putty.exe") {
		command = puttySSH(c, secret, currentPath)
	} else if strings.HasPrefix(c, "xshell.exe") {
		command = xshellSSH(c)
	} else {
		global.LOG.Error("Type not supported")
	}
	return exec.Command(command)
}

func puttySSH(c string, secret string, currentPath string) string {
	c = strings.TrimPrefix(c, "putty.exe ")
	puttyPath := "putty.exe"
	if _, err := exec.LookPath("putty.exe"); err != nil {
		puttyPath = filepath.Join(currentPath, "putty.exe")
	}
	return fmt.Sprintf("%s %s -pw %s", puttyPath, c, secret)
}

func xshellSSH(c string) string {
	return c
}

func structurePostgreSQLCommand(command string) string {
	command = strings.Trim(strings.ReplaceAll(command, "psql ", ""), `"`)
	db := &DBCommand{}
	for _, v := range strings.Split(command, " ") {
		tp, val := strings.Split(v, "=")[0], strings.Split(v, "=")[1]
		switch tp {
		case "user":
			db.User = val
		case "password":
			db.Password = val
		case "host":
			db.Host = val
		case "port":
			db.Port = val
		case "dbname":
			db.DBName = val
		}
	}
	command = fmt.Sprintf(
		`psql "user=%s password=%s host=%s dbname=%s port=%s"`,
		db.User, db.Password, db.Host, db.DBName, db.Port,
	)
	return command
}
func handleDB(command string) *exec.Cmd {
	cmd := exec.Command("cmd")
	cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true, CmdLine: `/c start cmd /k ` + command}
	return cmd
}
