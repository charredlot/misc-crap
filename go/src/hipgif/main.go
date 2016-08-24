package main

import (
	"errors"
	"flag"
	"fmt"
	"image"
	"image/color"
	"image/gif"
	"os"
)

var (
	debug     bool
	macroName string
)

func cropFrame(frame *image.Paletted, rect image.Rectangle) (*image.Paletted,
	error) {
	var newFrame image.Paletted

	targetRect := rect.Intersect(frame.Rect)

	if debug {
		fmt.Printf("%+v len %d stride %d target %+v\n", frame.Rect,
			len(frame.Pix), frame.Stride, targetRect)
	}

	if targetRect.Empty() {
		return nil, nil
	}

	// translate to (0, 0)
	newFrame.Rect = targetRect.Sub(rect.Min)
	newFrame.Stride = newFrame.Rect.Max.X - newFrame.Rect.Min.X
	newFrame.Pix = make([]uint8, newFrame.Stride*
		(newFrame.Rect.Max.Y-newFrame.Rect.Min.Y))
	newFrame.Palette = frame.Palette

	transparent := frame.Palette.Index(&color.RGBA{255, 255, 255, 1})
	for x := targetRect.Min.X; x < targetRect.Max.X; x++ {
		for y := targetRect.Min.Y; y < targetRect.Max.Y; y++ {
			var pix uint8

			p := image.Point{x, y}
			if p.In(frame.Rect) {
				pix = frame.Pix[(y-frame.Rect.Min.Y)*frame.Stride+
					(x-frame.Rect.Min.X)]
			} else {
				pix = uint8(transparent)
			}
			newFrame.Pix[(y-targetRect.Min.Y)*newFrame.Stride+
				(x-targetRect.Min.X)] = pix
		}
	}

	return &newFrame, nil
}

type getBounds func(g *gif.GIF) image.Rectangle

func cropGIF(g *gif.GIF, rect image.Rectangle) (*gif.GIF, error) {
	var newGif gif.GIF

	newGif = *g
	newGif.Image = []*image.Paletted{}
	newGif.Delay = []int{}
	newGif.Disposal = []byte{}

	for i := range g.Image {
		newFrame, err := cropFrame(g.Image[i], rect)
		if err != nil {
			return nil, err
		}

		if newFrame == nil {
			if debug {
				fmt.Println("  empty frame after cropping")
			}
		} else {
			newGif.Image = append(newGif.Image, newFrame)
			newGif.Delay = append(newGif.Delay, g.Delay[i])
			newGif.Disposal = append(newGif.Disposal, g.Disposal[i])

			if debug {
				fmt.Printf("  %+v len %d stride %d\n", newFrame.Rect,
					len(newFrame.Pix), newFrame.Stride)
			}
		}
	}

	newGif.Config.Width = rect.Dx()
	newGif.Config.Height = rect.Dy()
	return &newGif, nil
}

func outFilename(index int) string {
	return fmt.Sprintf("./tmp%d.gif", index)
}

func cropAndWriteGIF(g *gif.GIF, rect image.Rectangle, index int) error {
	filename := outFilename(index)
	out, err := os.Create(filename)
	if err != nil {
		return errors.New(fmt.Sprintf("open error %s %v", filename, err))
	}
	defer out.Close()

	edited, err := cropGIF(g, rect)
	if err != nil {
		return errors.New(fmt.Sprintf("crop error %s %v", filename, err))
	}

	if debug {
		fmt.Printf("%d new dim %d x %d\n", index,
			edited.Config.Width, edited.Config.Height)
	}
	err = gif.EncodeAll(out, edited)
	if err != nil {
		return errors.New(fmt.Sprintf("encode error %s %v", filename, err))
	}

	fmt.Fprintf(os.Stderr, "wrote %s\n", filename)

	return nil
}

func getTiles(maxWidth, maxHeight int) ([]image.Rectangle, error) {
	var rects []image.Rectangle

	side := maxHeight / 2

	filename := "./tmp.html"
	html, err := os.Create(filename)
	if err != nil {
		return nil, err
	}
	defer html.Close()

	fmt.Fprintf(os.Stderr, "preview file %s\n", filename)
	fmt.Fprintln(os.Stderr, "hipchat image macro:")

	html.WriteString("<html><body>\n")
	for y := 0; y < maxHeight; y += side {
		for x := 0; x < maxWidth; x += side {
			var rect image.Rectangle

			rect.Min.X = x
			rect.Max.X = x + side
			if rect.Max.X > maxWidth {
				rect.Max.X = maxWidth
			}
			rect.Min.Y = y
			rect.Max.Y = y + side
			if rect.Max.Y > maxHeight {
				rect.Max.Y = maxHeight
			}

			imgFilename := outFilename(len(rects))
			html.WriteString(fmt.Sprintf("<img src=\"%s\">\n", imgFilename))
			fmt.Fprintf(os.Stderr, "(%s%d)", macroName, len(rects))
			rects = append(rects, rect)
		}
		html.WriteString("<br>\n")
		fmt.Fprintf(os.Stderr, "\n")
	}
	html.WriteString("</body></html>\n")

	return rects, nil
}

func tileGIF(g *gif.GIF) error {
	// assume first image has full bounds of image
	rects, err := getTiles(g.Config.Width, g.Config.Height)
	if err != nil {
		return nil
	}

	for i := range rects {
		err := cropAndWriteGIF(g, rects[i], i)
		if err != nil {
			return err
		}
	}

	return nil
}

func main() {
	flag.BoolVar(&debug, "v", false, "verbose")
	flag.StringVar(&macroName, "macro", "img",
		"name for image macro in hipchat")
	flag.Parse()

	filename := flag.Args()[0]
	f, err := os.Open(filename)
	if err != nil {
		fmt.Println("open error", filename, err)
		return
	}
	defer f.Close()

	g, err := gif.DecodeAll(f)
	if err != nil {
		fmt.Println("gif error", err)
		return
	}
	fmt.Fprintf(os.Stderr, "frames %d width %d x height %d\n",
		len(g.Image), g.Config.Width, g.Config.Height)

	err = tileGIF(g)
	if err != nil {
		fmt.Println("tile error", err)
	}
}
