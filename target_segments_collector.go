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
			_, err := writer.WriteString(scanner.Text())
			return err
		}
		if state.Kind == SEGMENT_NOT_DEFINED ||
			(c.targetSegment == DEFAULT_TARGET_SEGMENT && state.Targets == "") ||
			strings.Contains(state.Targets, c.targetSegment) {
			if _, err := writer.WriteString(scanner.Text()); err != nil {
				return err
			}
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
