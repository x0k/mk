package main

import (
	"reflect"
	"strings"
	"testing"
)

func TestSegmentsScanner(t *testing.T) {
	tests := []struct {
		name     string
		content  string
		segments []SegmentsScannerState
		texts    []string
	}{
		{
			name: "Compact scan",
			content: `foo: bar baz
	foo-content
common-content
xxx: !
	excluded
bar:
	bar-content
mk:`,
			segments: []SegmentsScannerState{
				{SEGMENT_CONTINUED, "foo", []string{"bar", "baz"}},
				{SEGMENT_NOT_DEFINED, "", nil},
				{SEGMENT_CONTINUED, "xxx", []string{"!"}},
				{SEGMENT_CONTINUED, "bar", nil},
				{SEGMENT_STARTS, "mk", nil},
			},
			texts: []string{
				"foo-content\n",
				"common-content\n",
				"excluded\n",
				"bar-content\n",
				"",
			},
		},
		{
			name: "Spaces scan",
			content: `#!/bin/bash -xe

foo: !
	line-1
	line-2

mk:

`,
			segments: []SegmentsScannerState{
				{SEGMENT_NOT_DEFINED, "", nil},
				{SEGMENT_CONTINUED, "foo", []string{"!"}},
				{SEGMENT_NOT_DEFINED, "", nil},
				{SEGMENT_STARTS, "mk", nil},
				{SEGMENT_NOT_DEFINED, "", nil},
			},
			texts: []string{
				"#!/bin/bash -xe\n\n",
				"line-1\nline-2\n",
				"\n",
				"",
				"\n",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			scn := NewSegmentsScanner(strings.NewReader(tt.content))
			for i := 0; scn.Scan(); i++ {
				if i >= len(tt.segments) {
					t.Errorf("Too many segments")
					return
				}
				expectedSegment := tt.segments[i]
				state := scn.State()
				text := scn.Text()
				if state.Kind != expectedSegment.Kind {
					t.Errorf("Expected segment kind %d, got %d", expectedSegment.Kind, state.Kind)
				}
				if state.Segment != expectedSegment.Segment {
					t.Errorf("Expected segment %q, got %q", expectedSegment.Segment, state.Segment)
				}
				if (len(state.Targets) > 0 || len(expectedSegment.Targets) > 0) &&
					!reflect.DeepEqual(state.Targets, expectedSegment.Targets) {
					t.Errorf("Expected targets %q, got %q", expectedSegment.Targets, state.Targets)
				}
				if text != tt.texts[i] {
					t.Errorf("Expected text %q, got %q", tt.texts[i], text)
				}
			}
		})
	}
}
