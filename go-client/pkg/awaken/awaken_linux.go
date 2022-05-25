package awaken

import (
	"fmt"
	"os/exec"
)

func awakenCommand(command string) *exec.Cmd {
	cmd := exec.Command(
		"gnome-terminal", "--", "bash", "-c",
		fmt.Sprintf("%s; exec bash -i", command),
	)
	return cmd
}
