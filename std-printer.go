package main

import (
	"fmt"
)

type stdRecipeLinesPrinter struct{}

var StdRecipeLinesPrinter = &stdRecipeLinesPrinter{}

func (p *stdRecipeLinesPrinter) Print(lines string) error {
	_, err := fmt.Print(lines)
	return err
}
