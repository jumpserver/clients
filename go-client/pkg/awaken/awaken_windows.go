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

func handleSSH(t *Token, currentPath string) *exec.Cmd {
	puttyPath := "putty.exe"
	if _, err := exec.LookPath("putty.exe"); err != nil {
		puttyPath = filepath.Join(currentPath, "putty.exe")
	}
	cmd := exec.Command(
		puttyPath, "-ssh",
		fmt.Sprintf("%s@%s", t.UserName, t.Ip), "-P", t.Port, "-pw", t.Password,
	)
	return cmd
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
