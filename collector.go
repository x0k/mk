package main

import (
	"regexp"
	"strings"
)

const (
	recipe_not_defined   = 0
	define_recipe_indent = 1
	recipe_defined       = 2
)

var recipeNameRegExp = regexp.MustCompile(`^[A-Za-z][0-9A-Za-z\t _-]*:$`)
var recipeIndentRegExp = regexp.MustCompile(`^([ \t]+)`)

type recipeLinesCollector struct {
	state             int
	targetRecipe      string
	isRecipeFound     bool
	recipeIndentation string
	lines             string
}

func NewRecipeLinesCollector(targetRecipe string) RecipeLinesCollector {
	return &recipeLinesCollector{
		targetRecipe: targetRecipe,
	}
}

func (r *recipeLinesCollector) appendLine(line string) {
	r.lines = r.lines + line + "\n"
}

func (r *recipeLinesCollector) appendRecipeLine(line string) {
	if r.isRecipeFound {
		r.appendLine(strings.TrimPrefix(line, r.recipeIndentation))
	}
}

func (r *recipeLinesCollector) setRecipe(line string) {
	r.isRecipeFound = r.targetRecipe == line[:len(line)-1]
	r.state = define_recipe_indent
}

func (r *recipeLinesCollector) handleRecipeChange(line string) error {
	if r.isRecipeFound {
		return RLCDoneError
	}
	if recipeNameRegExp.MatchString(line) {
		r.setRecipe(line)
	} else {
		r.appendLine(line)
		r.state = recipe_not_defined
	}
	return nil
}

func (r *recipeLinesCollector) WriteString(line string) (int, error) {
	switch r.state {
	case recipe_not_defined:
		if recipeNameRegExp.MatchString(line) {
			r.setRecipe(line)
		} else {
			r.appendLine(line)
		}
	case define_recipe_indent:
		matches := recipeIndentRegExp.FindStringSubmatch(line)
		if len(matches) == 2 {
			r.recipeIndentation = matches[1]
			r.appendRecipeLine(line)
			r.state = recipe_defined
		} else if err := r.handleRecipeChange(line); err != nil {
			return 0, err
		}
	case recipe_defined:
		if strings.HasPrefix(line, r.recipeIndentation) {
			r.appendRecipeLine(line)
		} else if err := r.handleRecipeChange(line); err != nil {
			return 0, err
		}
	}
	return len(line), nil
}

func (r *recipeLinesCollector) GetLines() string {
	return r.lines
}

func (r *recipeLinesCollector) IsRecipeFound() bool {
	return r.isRecipeFound
}
