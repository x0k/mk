package main

import (
	"bufio"
	"strings"
)

type segmentLinesCollector struct {
	state                int
	targetSegment        string
	isCollectableSegment bool
	isTargetSegmentFound bool
	segmentIndentation   string
	lines                string
}

func NewSegmentLinesCollector(targetSegment string) LinesCollector {
	return &segmentLinesCollector{
		targetSegment: targetSegment,
	}
}

func (r *segmentLinesCollector) appendLine(line string) {
	r.lines = r.lines + line + "\n"
}

func (r *segmentLinesCollector) appendSegmentLine(line string) {
	if r.isCollectableSegment || r.isTargetSegmentFound {
		r.appendLine(strings.TrimPrefix(line, r.segmentIndentation))
	}
}

func (r *segmentLinesCollector) startSegment(matched []string) {
	r.isTargetSegmentFound = r.targetSegment == matched[1]
	if !r.isTargetSegmentFound && len(matched) > 2 {
		targets := strings.Fields(matched[2])
		for _, target := range targets {
			if r.targetSegment == target {
				r.isCollectableSegment = true
				break
			}
		}
	}
	r.state = SEGMENT_STARTS
}

func (r *segmentLinesCollector) finishSegment(line string) {
	if r.isTargetSegmentFound {
		r.state = TARGET_SEGMENT_FINISHED
		return
	}
	r.isCollectableSegment = false
	matched := SEGMENT_NAME_REG_EXP.FindStringSubmatch(line)
	if matched != nil {
		r.startSegment(matched)
	} else {
		r.appendLine(line)
		r.state = SEGMENT_NOT_DEFINED
	}
}

func (r *segmentLinesCollector) collectLine(line string) {
	switch r.state {
	case SEGMENT_NOT_DEFINED:
		matched := SEGMENT_NAME_REG_EXP.FindStringSubmatch(line)
		if matched != nil {
			r.startSegment(matched)
		} else {
			r.appendLine(line)
		}
	case SEGMENT_STARTS:
		matches := SEGMENT_INDENT_REG_EXP.FindStringSubmatch(line)
		if matches != nil {
			r.segmentIndentation = matches[1]
			r.appendSegmentLine(line)
			r.state = SEGMENT_CONTINUED
		} else {
			r.finishSegment(line)
		}
	case SEGMENT_CONTINUED:
		if strings.HasPrefix(line, r.segmentIndentation) {
			r.appendSegmentLine(line)
		} else {
			r.finishSegment(line)
		}
	}
}

func (r *segmentLinesCollector) CollectLines(scanner *bufio.Scanner) (bool, error) {
	for scanner.Scan() {
		r.collectLine(scanner.Text())
		if r.state == TARGET_SEGMENT_FINISHED {
			return true, nil
		}
	}
	if err := scanner.Err(); err != nil {
		return false, err
	}
	return r.isTargetSegmentFound, nil
}

func (r *segmentLinesCollector) GetLines() string {
	return r.lines
}
