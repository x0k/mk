package main

import "bufio"

type LinesCollector interface {
	CollectLines(scanner *bufio.Scanner) (bool, error)
	GetLines() string
}

type LinesPrinter interface {
	Print(lines string) error
}
