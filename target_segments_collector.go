package main

import (
	"io"
	"strings"
)

type targetSegmentsCollector struct {
	targetSegment string
}

func NewTargetSegmentsCollector(targetSegment string) *targetSegmentsCollector {
	return &targetSegmentsCollector{targetSegment: targetSegment}
}

type SegmentsScanner interface {
	Scan() bool
	Err() error
	Text() string
	State() SegmentsScannerState
}

func (c *targetSegmentsCollector) Collect(scanner SegmentsScanner, writer io.StringWriter) error {
	for scanner.Scan() {
		state := scanner.State()
		if state.Segment == c.targetSegment {
			writer.WriteString(scanner.Text())
			return nil
		}
		if state.Kind == SEGMENT_NOT_DEFINED ||
			(c.targetSegment == DEFAULT_TARGET_SEGMENT && !state.ExcludeDefaultTarget) ||
			strings.Contains(state.Targets, c.targetSegment) {
			writer.WriteString(scanner.Text())
		}
	}
	if err := scanner.Err(); err != nil {
		return err
	}
	if c.targetSegment != DEFAULT_TARGET_SEGMENT {
		return ErrSegmentNotFound
	}
	return nil
}
