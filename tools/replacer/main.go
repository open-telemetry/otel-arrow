package main

import (
	"bufio"
	"bytes"
	"fmt"
	"log"
	"os"
	"path"
	"regexp"
	"sort"
	"strings"
)

func findMods(modPath string, dir *os.File) ([]string, error) {
	names, err := dir.Readdirnames(-1)
	if err != nil {
		return nil, fmt.Errorf("readdirnames %q: %w", modPath, err)
	}
	var allMods []string

	for _, name := range names {
		childPath := path.Join(modPath, name)

		if name == "go.mod" {
			allMods = append(allMods, childPath)
		}
		child, err := os.Open(childPath)
		if err != nil {
			return nil, fmt.Errorf("open %q: %w", childPath, err)
		}

		fi, err := child.Stat()
		if err != nil {
			return nil, fmt.Errorf("stat %q: %w", childPath, err)
		}
		if !fi.IsDir() {
			continue

		}

		childMods, err := findMods(childPath, child)
		if err != nil {
			return nil, err
		}
		allMods = append(allMods, childMods...)
	}
	return allMods, nil
}

var moduleRe = regexp.MustCompile(`(?m:^module (.*)$)`)
var replaceRe = regexp.MustCompile(`(?m:^\s*(//+)?\s*replace\s*(.*)$)`)

func moduleName(modPath string) (string, []byte, error) {
	data, err := os.ReadFile(modPath)
	if err != nil {
		return "", nil, fmt.Errorf("read %q: %w", modPath, err)
	}
	mod := moduleRe.FindSubmatch(data)
	if mod == nil {
		return "", nil, fmt.Errorf("no module in %s", modPath)
	}

	return string(mod[1]), data, nil
}

func main() {
	err := Main()
	if err != nil {
		log.Println(err)
	}
}

func Main() error {
	if len(os.Args) != 2 {
		return fmt.Errorf("usage: %s [fix|unfix]\n", os.Args[0])
	}
	mode := os.Args[1]
	switch mode {
	case "fix", "unfix":
	default:
		return fmt.Errorf("usage: %s [fix|unfix]\n", os.Args[0])
	}

	_, err := os.ReadFile("go.work")
	if err != nil {
		return fmt.Errorf("there must be a go.work in the current dir")
	}

	dot, err := os.Open(".")
	if err != nil {
		return fmt.Errorf("open '.': %w", err)
	}

	allPaths, err := findMods(".", dot)
	if err != nil {
		return fmt.Errorf("find modules: %w", err)
	}
	sort.Strings(allPaths)

	var allNames []string
	var allData [][]byte
	for _, modPath := range allPaths {
		name, data, err := moduleName(modPath)
		if err != nil {
			return fmt.Errorf("module %s: %w", modPath, err)
		}
		allNames = append(allNames, name)
		allData = append(allData, data)
	}

	for idx := range allPaths {
		modPath := allPaths[idx]
		data := allData[idx]
		name := allNames[idx]

		if err := fixModule(mode, name, modPath, data, allNames, allPaths); err != nil {
			return fmt.Errorf("module %s: %w", modPath, err)
		}
	}
	return nil
}

func fixModule(mode, name, modPath string, data []byte, allNames, allPaths []string) error {
	var output bytes.Buffer

	sc := bufio.NewScanner(strings.NewReader(string(data)))
	for sc.Scan() {
		replace := replaceRe.FindStringSubmatch(sc.Text())
		if replace == nil {
			output.WriteString(sc.Text())
			output.WriteString("\n")
			continue
		}
	}
	if err := sc.Err(); err != nil {
		return err
	}

	if mode == "fix" {
		for idx := range allNames {
			if allNames[idx] == name {
				continue
			}
			output.WriteString("replace ")
			output.WriteString(allNames[idx])
			output.WriteString(" => ")
			output.WriteString(modReplacement(modPath, allPaths[idx]))
			output.WriteString("\n")
		}
	}

	tmpFile := modPath + ".bak"
	if err := os.WriteFile(tmpFile, output.Bytes(), 0666); err != nil {
		return fmt.Errorf("write %q: %w", tmpFile, err)
	}
	if err := os.Rename(tmpFile, modPath); err != nil {
		return fmt.Errorf("rename %q->%q: %w", tmpFile, modPath, err)
	}

	return nil
}

func modReplacement(from, to string) string {
	repl := "./"
	from, _ = path.Split(from)
	from = path.Clean(from)
	to, _ = path.Split(to)
	to = path.Clean(to)

	for len(from) > 1 {
		from, _ = path.Split(from)
		from = path.Clean(from)
		repl = "../" + repl
	}

	repl += to
	repl = path.Clean(repl)
	if strings.HasPrefix(repl, "../") {
		return repl
	}
	return "./" + repl
}
