package main

import (
	"bufio"
	"io"
	"log"
	"os"
	"strings"
)

type ReceiptLinesCollector interface {
	io.StringWriter
	GetLines() []string
	IsInTargetReceipt() bool
}

type ReceiptLinesCollectorDoneError struct{}

var RLCDoneError = &ReceiptLinesCollectorDoneError{}

func (e ReceiptLinesCollectorDoneError) Error() string {
	return "done"
}

type ReceiptLinesPrinter interface {
	Print(lines []string) error
}

func collectReceiptLines(collector ReceiptLinesCollector, scanner *bufio.Scanner) (bool, error) {
	for scanner.Scan() {
		if _, err := collector.WriteString(scanner.Text()); err != nil {
			if err == RLCDoneError {
				return true, nil
			}
			return false, err
		}
	}
	if err := scanner.Err(); err != nil {
		return false, err
	}
	return collector.IsInTargetReceipt(), nil
}

func makePrinter(firstLine string) (ReceiptLinesPrinter, error) {
	if strings.HasPrefix(firstLine, "#!") {
		return NewCmdReceiptLinesPrinter(firstLine)
	} else {
		return NewStdReceiptLinesPrinter(), nil
	}
}

func main() {
	if len(os.Args) < 2 {
		log.Fatal("No receipt name provided")
	}
	file, err := os.Open("receipts")
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	receiptName := os.Args[1]
	collector := NewReceiptLinesCollector(receiptName)
	isReceiptFounded, err := collectReceiptLines(collector, scanner)
	if err != nil {
		log.Fatal("Error during collection receipt lines", err)
	}
	if !isReceiptFounded {
		log.Fatalf("Receipt \"%s\" not found", receiptName)
	}
	lines := collector.GetLines()
	if len(lines) < 1 {
		log.Fatal("Receipts file is empty")
	}
	printer, err := makePrinter(lines[0])
	if err != nil {
		log.Fatal("Error during creating printer", err)
	}
	err = printer.Print(collector.GetLines())
	if err != nil {
		log.Fatal("Error during printing", err)
	}
}
