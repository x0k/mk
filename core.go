package main

import (
	"bufio"
	"regexp"
)

type LinesCollector interface {
	CollectLines(scanner *bufio.Scanner) (bool, error)
	Lines() string
}

type LinesPrinter interface {
	Print(lines string) error
}

const (
	DEFAULT_TARGET_SEGMENT  = "all"
	SEGMENT_NOT_DEFINED     = 0
	SEGMENT_STARTS          = 1
	SEGMENT_CONTINUED       = 2
	TARGET_SEGMENT_FINISHED = 3
)

var SEGMENT_NAME_REG_EXP = regexp.MustCompile(`^([A-z][0-9A-z_-]*):(.*)$`)
var SEGMENT_INDENT_REG_EXP = regexp.MustCompile(`^([ \t]+)`)
