package main

import (
	"os"
	"os/exec"
)

type cmdRecipeLinesPrinter struct {
}

var CmdRecipeLinesPrinter = &cmdRecipeLinesPrinter{}

// Print implements RecipeLinesPrinter.
func (p *cmdRecipeLinesPrinter) Print(lines string) error {
	tmpFile, err := os.CreateTemp("", "cook_tmp_script_*")
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

	cmd := exec.Command(tmpFile.Name())
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = os.Environ()
	return cmd.Run()
}
