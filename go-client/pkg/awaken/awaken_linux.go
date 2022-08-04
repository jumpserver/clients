package awaken

import (
	"fmt"
	"go-client/global"
	"os/exec"
	"strings"
)

func awakenRDPCommand(filePath string) *exec.Cmd {
	global.LOG.Debug(filePath)
	cmd := exec.Command("remmina", filePath)
	return cmd
}

func awakenCommand(command string) *exec.Cmd {
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
		cmd = exec.Command("deepin-terminal", "-C", command)
	default:
		msg := fmt.Sprintf("Not yet supported %s desktop system", currentDesktop)
		global.LOG.Info(msg)
	}
	return cmd
}
