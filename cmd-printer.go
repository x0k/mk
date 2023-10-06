package main

import (
	"errors"
	"io"
	"os"
	"os/exec"
	"strings"
)

type cmdReceiptLinesPrinter struct {
	cmd *exec.Cmd
}

func NewCmdReceiptLinesPrinter(shebangLine string) (ReceiptLinesPrinter, error) {
	parts := strings.Fields(shebangLine[2:])
	if len(parts) < 1 {
		return nil, errors.New("invalid shebang line")
	}
	cmd := exec.Command(parts[0], parts[1:]...)
	// cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = os.Environ()
	return &cmdReceiptLinesPrinter{
		cmd: cmd,
	}, nil
}

// Print implements ReceiptLinesPrinter.
func (p *cmdReceiptLinesPrinter) Print(lines []string) error {
	// Create a pipe for stdin.
	stdin, err := p.cmd.StdinPipe()
	if err != nil {
		return err
	}
	defer stdin.Close()

	if err := p.cmd.Start(); err != nil {
		return err
	}
	for _, line := range lines {
		_, err := io.WriteString(stdin, line+"\n")
		if err != nil {
			return err
		}
	}
	err = stdin.Close()
	if err != nil {
		return err
	}
	return p.cmd.Wait()
}
