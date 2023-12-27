package main

import (
	"bufio"
	"os"
)

type segmentsWriter struct {
	written bool
	writer  *bufio.Writer
}

func NewSegmentsWriter() *segmentsWriter {
	return &segmentsWriter{
		writer: bufio.NewWriter(os.Stdout),
	}
}

func (w *segmentsWriter) WriteString(s string) (int, error) {
	if w.written {
		err := w.writer.WriteByte(' ')
		if err != nil {
			return 0, err
		}
	} else {
		w.written = true
	}
	n, err := w.writer.WriteString(s)
	if err != nil {
		return 0, err
	}
	if w.written {
		return n + 1, nil
	}
	return n, nil
}

func (w *segmentsWriter) Flush() error {
	return w.writer.Flush()
}
