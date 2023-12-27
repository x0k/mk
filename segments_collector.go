package main

import "io"

type segmentsCollector struct{}

func NewSegmentsCollector() *segmentsCollector {
	return &segmentsCollector{}
}

func (c *segmentsCollector) Collect(scanner SegmentsScanner, writer io.StringWriter) error {
	for scanner.Scan() {
		state, segment, _ := scanner.State()
		if state == SEGMENT_NOT_DEFINED {
			continue
		}
		writer.WriteString(segment)
	}
	return scanner.Err()
}
