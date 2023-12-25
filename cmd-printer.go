package main

import (
	"os"
	"os/exec"
)

type cmdLinesPrinter struct {
	args []string
}

func NewCmdLinesPrinter(args []string) *cmdLinesPrinter {
	return &cmdLinesPrinter{
		args: args,
	}
}

func (p *cmdLinesPrinter) Print(lines string) error {
	tmpFile, err := os.CreateTemp("", "mk_tmp_script_*")
	if err != nil {
		return err
	}
	defer os.Remove(tmpFile.Name())
	defer tmpFile.Close()
	_, err = tmpFile.WriteString(lines)
	if err != nil {
		return err
	}
	err = os.Chmod(tmpFile.Name(), 0755)
	if err != nil {
		return err
	}
	err = tmpFile.Close()
	if err != nil {
		return err
	}

	cmd := exec.Command(tmpFile.Name(), p.args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = os.Environ()
	return cmd.Run()
}
