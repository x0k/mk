package main

import (
	"fmt"
	"strings"
)

type stdReceiptLinesPrinter struct{}

func NewStdReceiptLinesPrinter() ReceiptLinesPrinter {
	return &stdReceiptLinesPrinter{}
}

func (p *stdReceiptLinesPrinter) Print(lines []string) error {
	_, err := fmt.Print(strings.Join(lines, "\n"))
	return err
}
