package tile

import (
	"errors"
	"fmt"
	"image"
	"image/color"
	"image/gif"
	"io"
	"os"
)

type GetURI func(row int, col int) string

func cropFrame(frame *image.Paletted, rect image.Rectangle) (*image.Paletted,
	error) {
	var newFrame image.Paletted

	targetRect := rect.Intersect(frame.Rect)

	if Debug {
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
			if Debug {
				fmt.Println("  empty frame after cropping")
			}
		} else {
			newGif.Image = append(newGif.Image, newFrame)
			newGif.Delay = append(newGif.Delay, g.Delay[i])
			newGif.Disposal = append(newGif.Disposal, g.Disposal[i])

			if Debug {
				fmt.Printf("  %+v len %d stride %d\n", newFrame.Rect,
					len(newFrame.Pix), newFrame.Stride)
			}
		}
	}

	newGif.Config.Width = rect.Dx()
	newGif.Config.Height = rect.Dy()
	return &newGif, nil
}

func writeGIF(g *gif.GIF, filename string) error {
	out, err := os.Create(filename)
	if err != nil {
		return errors.New(fmt.Sprintf("open error %s %v", filename, err))
	}
	defer out.Close()

	err = gif.EncodeAll(out, g)
	if err != nil {
		return errors.New(fmt.Sprintf("encode error %s %v", filename, err))
	}

	fmt.Fprintf(os.Stderr, "wrote %s\n", filename)

	return nil
}

func WriteHTMLPreview(w io.Writer, rows, cols int, uri GetURI) error {
	for row := 0; row < rows; row++ {
		_, err := w.Write([]byte("<div>"))
		if err != nil {
			return err
		}

		for col := 0; col < cols; col++ {
			_, err := w.Write([]byte(
				fmt.Sprintf("<img src=\"%s\">", uri(row, col))))
			if err != nil {
				return err
			}
		}
		_, err = w.Write([]byte("</div>\n"))
		if err != nil {
			return err
		}
	}

	return nil
}

func writeHTMLPreviewToFile(filename string, rows, cols int, uri GetURI) error {
	html, err := os.Create(filename)
	if err != nil {
		return errors.New(fmt.Sprintf(
			"error creating preview file %s %s\n", filename, err))
	}
	defer html.Close()

	html.WriteString("<html><body>\n")
	WriteHTMLPreview(html, rows, cols, uri)
	html.WriteString("</body></html>\n")
	return nil
}

func getTiles(maxWidth, maxHeight int) ([][]image.Rectangle, error) {
	var rects [][]image.Rectangle

	side := maxHeight / 2

	for y := 0; y < maxHeight; y += side {
		row := []image.Rectangle{}
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

			row = append(row, rect)
		}
		rects = append(rects, row)
	}

	return rects, nil
}

func TileGIF(g *gif.GIF) ([][]*gif.GIF, error) {
	rects, err := getTiles(g.Config.Width, g.Config.Height)
	if err != nil {
		return nil, err
	}

	var gifs [][]*gif.GIF
	for row := range rects {
		currRow := []*gif.GIF{}
		for col := range rects[row] {
			cropped, err := cropGIF(g, rects[row][col])
			if err != nil {
				return nil, err
			}

			if Debug {
				fmt.Printf("new dim %d x %d\n", cropped.Config.Width,
					cropped.Config.Height)
			}

			currRow = append(currRow, cropped)
		}
		gifs = append(gifs, currRow)
	}

	return gifs, nil
}

func TileGIFToFiles(g *gif.GIF) error {
	gifs, err := TileGIF(g)
	if err != nil {
		return err
	}

	rows := len(gifs)
	cols := len(gifs[0])
	getFilename := func(row, col int) string {
		return fmt.Sprintf("./tmp%d.gif", (row*cols)+col)
	}

	err = writeHTMLPreviewToFile("./tmp.html", rows, cols, getFilename)
	if err != nil {
		return err
	}
	fmt.Fprintf(os.Stderr, "wrote preview file tmp.html\n")

	fmt.Fprintf(os.Stderr, "hipchat macro:\n")
	for row := 0; row < rows; row++ {
		for col := 0; col < cols; col++ {
			fmt.Fprintf(os.Stderr, "(%s%d)", MacroName, (row*cols)+col)
		}
		fmt.Fprintf(os.Stderr, "\n")
	}

	for row := 0; row < rows; row++ {
		for col := 0; col < cols; col++ {
			err := writeGIF(gifs[row][col], getFilename(row, col))
			if err != nil {
				return err
			}
		}
	}

	return nil
}
