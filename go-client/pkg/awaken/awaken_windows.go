package awaken

import (
	"fmt"
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
	puttyPath := "putty.exe"
	if _, err := exec.LookPath("putty.exe"); err != nil {
		puttyPath = filepath.Join(currentPath, "putty.exe")
	}

	//TODO core api 有时判断系统会判断错 导致返回无putty
	if strings.HasPrefix(c, "putty.exe") {
		c = strings.Replace(c, "putty.exe -", "", 1)
	}

	c = strings.Replace(c, " -p ", " -P ", 1)
	c = fmt.Sprintf("-%s -pw %s", c, secret)
	command := strings.Split(c, " ")
	return exec.Command(puttyPath, command...)
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
