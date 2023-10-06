package main

import (
	"fmt"
)

type stdReceiptLinesPrinter struct{}

func NewStdReceiptLinesPrinter() ReceiptLinesPrinter {
	return &stdReceiptLinesPrinter{}
}

func (p *stdReceiptLinesPrinter) Print(lines string) error {
	_, err := fmt.Print(lines)
	return err
}
