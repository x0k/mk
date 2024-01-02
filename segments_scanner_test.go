package main

import (
	"strings"
	"testing"
)

func TestCompactScan(t *testing.T) {
	scanner := NewSegmentsScanner(strings.NewReader(`foo: bar baz
	foo-content
common-content
xxx!:
	excluded
bar:
	bar-content
all:`))
	segments := []SegmentsScannerState{
		{SEGMENT_CONTINUED, "foo", " bar baz", false},
		{SEGMENT_NOT_DEFINED, "", "", false},
		{SEGMENT_CONTINUED, "xxx", "", true},
		{SEGMENT_CONTINUED, "bar", "", false},
		{SEGMENT_STARTS, "all", "", false},
	}
	texts := []string{
		"foo-content\n",
		"common-content\n",
		"excluded\n",
		"bar-content\n",
		"",
	}
	for i := 0; scanner.Scan(); i++ {
		if i >= len(segments) {
			t.Errorf("Too many segments")
			return
		}
		expectedSegment := segments[i]
		state := scanner.State()
		text := scanner.Text()
		if state.Kind != expectedSegment.Kind {
			t.Errorf("Expected state %d, got %d for segment %q (%d)", expectedSegment.Kind, state.Kind, expectedSegment.Segment, i)
		}
		if state.Segment != expectedSegment.Segment {
			t.Errorf("Expected segment %q, got %q", expectedSegment.Segment, state.Segment)
		}
		if state.Targets != expectedSegment.Targets {
			t.Errorf("Expected targets %q, got %q", expectedSegment.Targets, state.Targets)
		}
		if state.ExcludeDefaultTarget != expectedSegment.ExcludeDefaultTarget {
			t.Errorf("Expected exclude default target %t, got %t", expectedSegment.ExcludeDefaultTarget, state.ExcludeDefaultTarget)
		}
		if text != texts[i] {
			t.Errorf("Expected text %q, got %q", texts[i], text)
		}
	}
}

func TestSpacesScan(t *testing.T) {
	scanner := NewSegmentsScanner(strings.NewReader(`#!/bin/bash -xe

foo!:
	line-1
	line-2

all:

`))
	segments := []SegmentsScannerState{
		{SEGMENT_NOT_DEFINED, "", "", false},
		{SEGMENT_CONTINUED, "foo", "", true},
		{SEGMENT_NOT_DEFINED, "", "", false},
		{SEGMENT_STARTS, "all", "", false},
		{SEGMENT_NOT_DEFINED, "", "", false},
	}
	texts := []string{
		"#!/bin/bash -xe\n\n",
		"line-1\nline-2\n",
		"\n",
		"",
		"\n",
	}
	for i := 0; scanner.Scan(); i++ {
		if i >= len(segments) {
			t.Errorf("Too many segments")
			return
		}
		expectedSegment := segments[i]
		state := scanner.State()
		text := scanner.Text()
		if state.Kind != expectedSegment.Kind {
			t.Errorf("Expected state %d, got %d for segment %q", expectedSegment.Kind, state.Kind, expectedSegment.Segment)
		}
		if state.Segment != expectedSegment.Segment {
			t.Errorf("Expected segment %q, got %q", expectedSegment.Segment, state.Segment)
		}
		if state.Targets != expectedSegment.Targets {
			t.Errorf("Expected targets %q, got %q", expectedSegment.Targets, state.Targets)
		}
		if state.ExcludeDefaultTarget != expectedSegment.ExcludeDefaultTarget {
			t.Errorf("Expected exclude default target %t, got %t", expectedSegment.ExcludeDefaultTarget, state.ExcludeDefaultTarget)
		}
		if text != texts[i] {
			t.Errorf("Expected text %q, got %q", texts[i], text)
		}
	}
}
