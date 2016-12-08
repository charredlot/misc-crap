package main

import (
    "errors"
	"flag"
	"fmt"
	"html/template"
	"image/gif"
	"log"
	//    "mime/multipart"
	"net/http"
	"os"
    "strconv"
	"sync"
	"sync/atomic"

    "hipgif/tile"
)

const (
	maxFileSize = 20 * 1024 * 1024
)

var (
	uploadTemplate *template.Template
	previewTemplate *template.Template
	lastGIF        *gif.GIF
	cache          tiledGIFCache
	inc            int32
	debug          bool
)

type tiledGIF struct {
	key  string
    prefix string
	orig *gif.GIF
    tiled [][]*gif.GIF
}

type tiledGIFCache struct {
	last      [20]*tiledGIF
	nextIndex int
	m         map[string]*tiledGIF
	lock      sync.RWMutex
}

func (c *tiledGIFCache) Add(tg *tiledGIF) {
	c.lock.Lock()
	defer c.lock.Unlock()
	old := c.last[c.nextIndex]
	if old != nil {
		delete(c.m, old.key)
	}

	c.last[c.nextIndex] = tg
	c.m[tg.key] = tg
	c.nextIndex = (c.nextIndex + 1) % len(c.last)
}

func (c *tiledGIFCache) Get(key string) *tiledGIF {
	c.lock.RLock()
	defer c.lock.RUnlock()
	return c.m[key]
}

func returnError(w http.ResponseWriter, status int, msg string) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.WriteHeader(status)
	w.Write([]byte(msg))
}

func setupTemplates() error {
	var err error

	uploadTemplate, err = template.ParseFiles("templates/upload.html")
	if err != nil {
		return err
	}

	previewTemplate, err = template.ParseFiles("templates/preview.html")
	if err != nil {
		return err
	}

	return nil
}

func uploadView(w http.ResponseWriter, req *http.Request) {
	err := uploadTemplate.Execute(w, struct{}{})
	if err != nil {
		log.Println(err)
	}
}

func uploadFile(w http.ResponseWriter, req *http.Request) {
	if req.ContentLength > maxFileSize {
		returnError(w, http.StatusRequestEntityTooLarge,
			fmt.Sprintf("file too big: %d bytes", req.ContentLength))
		return
	}

	err := req.ParseMultipartForm(maxFileSize)
	if err != nil {
		log.Println(err)
		returnError(w, http.StatusBadRequest, "multipart parsing failed")
		return
	}

	err = req.ParseForm()
    if err != nil {
		log.Println(err)
		returnError(w, http.StatusBadRequest, "form parsing failed")
		return
    }

	// match "uploaded" with form in template
	uploaded, fh, err := req.FormFile("uploaded")
	if err != nil {
		log.Println("uploaded file error", err)
		returnError(w, http.StatusBadRequest, "messed up form you jerk")
		return
	}

	if debug {
		log.Printf("uploaded %s: %v\n", fh.Filename, fh.Header)
	}

	// XXX: check content-type?
	g, err := gif.DecodeAll(uploaded)
	if err != nil {
		returnError(w, http.StatusBadRequest,
			fmt.Sprintf("couldn't parse gif %s\n", err))
		return
	}

	id := atomic.AddInt32(&inc, 1)
	lastGIF = g

	if debug {
		log.Printf("caching %s with id %d\n", fh.Filename, id)
	}

    tiled, err := tile.TileGIF(g)
    if err != nil {
		returnError(w, http.StatusBadRequest,
            fmt.Sprintf("error tiling gif %s\n", err))
		return
    }

    // XXX: sanitize prefix
	tg := &tiledGIF{
        orig: g,
        key: fmt.Sprintf("%d", id),
        tiled: tiled,
        prefix: req.Form.Get("prefix"),
    }
	cache.Add(tg)

	http.Redirect(w, req, fmt.Sprintf("/preview?id=%d", id), http.StatusFound)
	/*
	       f, err := os.Create("tmp.gif")
	       if err != nil {
	   		log.Println(err)
	   		return
	       }
	       defer f.Close()


	       written, err := io.Copy(f, uploaded)
	       if err != nil {
	           log.Println(err)
	           return
	       }

	       log.Println("wrote", written, "bytes")
	       w.Write([]byte(fmt.Sprintf("uploaded %d bytes", written)))
	*/
}

func upload(w http.ResponseWriter, req *http.Request) {
	if req.Method == "GET" {
		uploadView(w, req)
	} else if req.Method == "POST" {
		uploadFile(w, req)
	} else {
		returnError(w, http.StatusNotAcceptable, "bad method")
	}
}

func getLast(w http.ResponseWriter, req *http.Request) {
	if req.Method != "GET" {
		returnError(w, http.StatusNotAcceptable, "bad method")
		return
	}

	if lastGIF == nil {
		returnError(w, http.StatusOK, "no last gif")
		return
	}

	w.Header().Set("Content-Type", "image/gif")
	w.WriteHeader(http.StatusOK)
	err := gif.EncodeAll(w, lastGIF)
	if err != nil {
		log.Println(err)
	}
}

func setupGet(w http.ResponseWriter, req *http.Request) error {
	if req.Method != "GET" {
		returnError(w, http.StatusNotAcceptable, "bad method")
		return errors.New("bad method")
	}

	err := req.ParseForm()
	if err != nil {
		returnError(w, http.StatusNotAcceptable, err.Error())
		return err
	}
    return nil
}

func getPreview(w http.ResponseWriter, req *http.Request) {
    if err := setupGet(w, req); err != nil {
		log.Println(err)
        return
    }

	id := req.Form.Get("id")
	tg := cache.Get(id)
	if tg == nil {
		returnError(w, http.StatusNotFound, fmt.Sprintf("%s not found", id))
		return
	}

    // XXX: should sanitize id maybe
    var ctx struct {
        ID string
        Tiled [][]*gif.GIF
        Prefix string
    }

    ctx.ID = id
    ctx.Tiled = tg.tiled
    ctx.Prefix = tg.prefix

	w.WriteHeader(http.StatusOK)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")

	err := previewTemplate.Execute(w, ctx)
	if err != nil {
		log.Println(err)
        returnError(w, http.StatusInternalServerError, err.Error())
        return
    }
}

func getTile(w http.ResponseWriter, req *http.Request) {
    if err := setupGet(w, req); err != nil {
		log.Println(err)
        return
    }

	id := req.Form.Get("id")
	row, err := strconv.Atoi(req.Form.Get("row"))
    if err != nil {
		returnError(w, http.StatusNotAcceptable, err.Error())
        return
    }

	col, err := strconv.Atoi(req.Form.Get("col"))
    if err != nil {
		returnError(w, http.StatusNotAcceptable, err.Error())
        return
    }

	if debug {
		log.Printf("got tiled id %s row %d col %d\n", id, row, col)
	}

	tg := cache.Get(id)
	if tg == nil {
		returnError(w, http.StatusNotFound, fmt.Sprintf("%s not found", id))
		return
	}

    g := tile.At(tg.tiled, row, col)
    if g == nil {
		returnError(w, http.StatusBadRequest,
            fmt.Sprintf("bad row %d or col %d", row, col))
		return
    }

	w.WriteHeader(http.StatusOK)
	w.Header().Set("Content-Type", "image/gif")
	err = gif.EncodeAll(w, g)
	if err != nil {
		log.Println(err)
		returnError(w, http.StatusBadRequest, "bad gif")
		return
	}
}

func getFull(w http.ResponseWriter, req *http.Request) {
    if err := setupGet(w, req); err != nil {
		log.Println(err)
        return
    }

	id := req.Form.Get("id")
	tg := cache.Get(id)
	if tg == nil {
		returnError(w, http.StatusNotFound, fmt.Sprintf("%s not found", id))
		return
	}

	w.WriteHeader(http.StatusOK)
	w.Header().Set("Content-Type", "image/gif")
	err := gif.EncodeAll(w, tg.orig)
	if err != nil {
		log.Println(err)
		returnError(w, http.StatusBadRequest, "bad gif")
		return
	}
}

func main() {
	var listen string

	flag.BoolVar(&debug, "v", true, "verbose")
	flag.StringVar(&listen, "l", ":8080", "host:port to listen on")
	flag.Parse()

	log.Println("Listening on", listen)

	if err := setupTemplates(); err != nil {
		log.Println("setup templates error", err)
		os.Exit(1)
	}

	cache.m = make(map[string]*tiledGIF)

	http.HandleFunc("/", upload)
	http.HandleFunc("/preview", getPreview)
	http.HandleFunc("/tile/", getTile)
	http.HandleFunc("/full", getFull)
	http.HandleFunc("/lastgif", getLast)
	http.ListenAndServe(listen, nil)
}
