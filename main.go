package main

import (
	"bufio"
	"log"
	"os"
	"strings"
)

func makePrinter(lines string, args []string) LinesPrinter {
	if strings.HasPrefix(lines, "#!") {
		return NewCmdLinesPrinter(args)
	} else {
		return StdLinesPrinter
	}
}

var fileNames = []string{"mkfile", "Mkfile"}

func main() {
	var file *os.File
	var err error
	for _, fileName := range fileNames {
		file, err = os.Open(fileName)
		if err == nil {
			defer file.Close()
			break
		}
	}
	if err != nil {
		log.Fatal("Mkfile not found, allowed file names: ", strings.Join(fileNames, ", "))
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	var collector LinesCollector
	targetSegment := "all"
	printerArgs := []string{}
	if len(os.Args) > 1 {
		targetSegment = os.Args[1]
		printerArgs = os.Args[2:]
	}
	collector = NewSegmentLinesCollector(targetSegment)
	isSegmentFound, err := collector.CollectLines(scanner)
	if err != nil {
		log.Fatal("Error during collecting segment lines ", err)
	}
	if !isSegmentFound {
		log.Fatalf("Segment \"%s\" not found ", targetSegment)
	}
	lines := collector.GetLines()
	if len(lines) < 1 {
		log.Fatal("Segment is empty")
	}
	printer := makePrinter(lines, printerArgs)
	if err != nil {
		log.Fatal("Error during creating printer ", err)
	}
	err = printer.Print(lines)
	if err != nil {
		log.Fatal("Error during printing ", err)
	}
}
