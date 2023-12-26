package main

import (
	"bufio"
	"log"
	"os"
	"strings"
)

func makeWriter(file *os.File, args []string) (BufferedWriter, error) {
	if strings.HasSuffix(file.Name(), "x") {
		return NewCmdWriter(args)
	}
	return bufio.NewWriter(os.Stdout), nil
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
	writer, err := makeWriter(file, args)
	if err != nil {
		log.Fatal("error during creating writer ", err)
	}
	err = NewTargetSegmentsCollector(targetSegment).Collect(NewSegmentsScanner(file), writer)
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
