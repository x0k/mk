package main

import (
	"bufio"
	"regexp"
)

type LinesCollector interface {
	CollectLines(scanner *bufio.Scanner) (bool, error)
	GetLines() string
}

type LinesPrinter interface {
	Print(lines string) error
}

const (
	SEGMENT_NOT_DEFINED     = 0
	SEGMENT_STARTS          = 1
	SEGMENT_CONTINUED       = 2
	TARGET_SEGMENT_FINISHED = 3
)

var SEGMENT_NAME_REG_EXP = regexp.MustCompile(`^[A-Za-z][0-9A-Za-z\t _-]*:$`)
var SEGMENT_INDENT_REG_EXP = regexp.MustCompile(`^([ \t]+)`)
