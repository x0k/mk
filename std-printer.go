package main

import (
	"fmt"
)

type stdRecipeLinesPrinter struct{}

func NewStdRecipeLinesPrinter() RecipeLinesPrinter {
	return &stdRecipeLinesPrinter{}
}

func (p *stdRecipeLinesPrinter) Print(lines string) error {
	_, err := fmt.Print(lines)
	return err
}
