package main

import (
	"bufio"
	"log"
	"os"
	"strings"
)

const SEGMENTS_FLAG = "--segments"

func makeServices(file *os.File, targetSegment string, args []string) (BufferedWriter, Collector, error) {
	if targetSegment == SEGMENTS_FLAG {
		return NewSegmentsWriter(), NewSegmentsCollector(), nil
	}
	if strings.HasSuffix(file.Name(), "x") {
		cmdWriter, err := NewCmdWriter(args)
		if err != nil {
			return nil, nil, err
		}
		return cmdWriter, NewTargetSegmentsCollector(targetSegment), nil
	}
	return bufio.NewWriter(os.Stdout), NewTargetSegmentsCollector(targetSegment), nil
}

func main() {
	var file *os.File
	var err error
	for _, fileName := range MK_FILE_NAMES {
		file, err = os.Open(fileName)
		if err == nil {
			defer file.Close()
			break
		}
	}
	if err != nil {
		log.Fatal("Mkfile not found, allowed file names: ", strings.Join(MK_FILE_NAMES, ", "))
	}
	targetSegment := DEFAULT_TARGET_SEGMENT
	args := []string{}
	if len(os.Args) > 1 {
		targetSegment = os.Args[1]
		args = os.Args[2:]
	}
	writer, collector, err := makeServices(file, targetSegment, args)
	if err != nil {
		log.Fatal("error during creating services ", err)
	}
	err = collector.Collect(NewSegmentsScanner(file), writer)
	if err == ErrSegmentNotFound {
		log.Fatalf("segment %q not found", targetSegment)
	}
	if err != nil {
		log.Fatal("error during collecting segments ", err)
	}
	err = writer.Flush()
	if err != nil {
		log.Fatal("error during printing ", err)
	}
}
