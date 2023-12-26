package main

import (
	"fmt"
)

type stdLinesPrinter struct{}

var StdLinesPrinter = &stdLinesPrinter{}

func (p *stdLinesPrinter) Print(lines string) error {
	_, err := fmt.Print(lines)
	return err
}
