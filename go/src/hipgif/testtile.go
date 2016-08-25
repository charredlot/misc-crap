package main

import (
	"flag"
	"fmt"
	"image/gif"
	"os"

	"hipgif/tile"
)

func main() {
	flag.BoolVar(&tile.Debug, "v", false, "verbose")
	flag.StringVar(&tile.MacroName, "macro", "img",
		"name for image macro in hipchat")
	flag.Parse()

	args := flag.Args()
	if len(args) == 0 {
		fmt.Printf("usage: %s $gif-filename\n\n", os.Args[0])
		flag.Usage()
		return
	}

	filename := args[0]
	f, err := os.Open(filename)
	if err != nil {
		fmt.Println("open error", filename, err)
		os.Exit(1)
	}
	defer f.Close()

	g, err := gif.DecodeAll(f)
	if err != nil {
		fmt.Println("gif error", err)
		os.Exit(1)
	}
	fmt.Fprintf(os.Stderr, "frames %d width %d x height %d\n",
		len(g.Image), g.Config.Width, g.Config.Height)

	err = tile.TileGIFToFiles(g)
	if err != nil {
		fmt.Println("tile error", err)
		os.Exit(1)
	}
}
