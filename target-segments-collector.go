package main

import (
	"io"
	"strings"
)

type targetSegmentsCollector struct {
	targetSegment string
	writer        io.StringWriter
}

func NewTargetSegmentsCollector(writer io.StringWriter, targetSegment string) *targetSegmentsCollector {
	return &targetSegmentsCollector{writer: writer, targetSegment: targetSegment}
}

type SegmentsScanner interface {
	Scan() bool
	Err() error
	Text() string
	State() (state SegmentsScannerState, segment string, targets string)
}

func (c *targetSegmentsCollector) Collect(scanner SegmentsScanner) (bool, error) {
	for scanner.Scan() {
		state, segment, targets := scanner.State()
		if segment == c.targetSegment {
			c.writer.WriteString(scanner.Text())
			return true, nil
		}
		if state == SEGMENT_NOT_DEFINED ||
			c.targetSegment == DEFAULT_TARGET_SEGMENT ||
			strings.Contains(targets, c.targetSegment) {
			c.writer.WriteString(scanner.Text())
		}
	}
	if err := scanner.Err(); err != nil {
		return false, err
	}
	return c.targetSegment == DEFAULT_TARGET_SEGMENT, nil
}
