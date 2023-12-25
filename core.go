package main

import (
	"errors"
	"regexp"
)

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
var ErrSegmentNotFound = errors.New("Segment not found")
