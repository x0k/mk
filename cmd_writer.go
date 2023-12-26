package main

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
)

type cmdWriter struct {
	bytes bytes.Buffer
	args  []string
}

func NewCmdWriter(args []string) (*cmdWriter, error) {
	return &cmdWriter{
		bytes: bytes.Buffer{},
		args:  args,
	}, nil
}

func (w *cmdWriter) WriteString(s string) (int, error) {
	return w.bytes.WriteString(s)
}

func (w *cmdWriter) Flush() error {
	file, err := os.CreateTemp("", "mk_tmp_file_*")
	if err != nil {
		return fmt.Errorf("error during create temp file: %w", err)
	}
	defer os.Remove(file.Name())
	defer file.Close()
	err = os.WriteFile(file.Name(), w.bytes.Bytes(), 0755)
	if err != nil {
		return fmt.Errorf("error during write: %w", err)
	}
	err = os.Chmod(file.Name(), 0755)
	if err != nil {
		return fmt.Errorf("error during chmod: %w", err)
	}
	err = file.Close()
	if err != nil {
		return fmt.Errorf("error during close: %w", err)
	}

	cmd := exec.Command(file.Name(), w.args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = os.Environ()
	return cmd.Run()
}
