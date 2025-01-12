package main

import (
	"bytes"
	"embed"
	"encoding/json"
	"fmt"
	"image"
	"image/png"
	"net/http"
	"os"
	"path/filepath"
	"slices"
	"strconv"
	"strings"

	"golang.org/x/image/draw"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/menu"
	"github.com/wailsapp/wails/v2/pkg/menu/keys"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

//go:embed all:frontend/dist
var assets embed.FS

// settings struct
type settings struct {
	LogLevel         string   `json:"log_level"`
	ImageDirectories []string `json:"image_directories"`
	WindowHeight     int      `json:"window_height"`
	WindowWidth      int      `json:"window_width"`
	WindowMaximized  bool     `json:"window_maximized"`
}

func main() {
	s, err := getSettings()
	if err != nil {
		println("Error getting settings:", err.Error())
		return
	}

	// Create an instance of the app structure
	app := NewApp(s)

	AppMenu := menu.NewMenu()
	FileMenu := AppMenu.AddSubmenu("File")
	FileMenu.AddText("Add folder", keys.CmdOrCtrl("a"), func(_ *menu.CallbackData) {
		imageDirectory, err := runtime.OpenDirectoryDialog(app.ctx, runtime.OpenDialogOptions{})
		if err != nil {
			println("Error opening image directory: ", err.Error())
		}
		if !slices.Contains(s.ImageDirectories, imageDirectory) {
			s.ImageDirectories = append(s.ImageDirectories, imageDirectory)
			configPath, _ := getSettingsPath()
			writeSettings(&s, configPath)
			// TODO: Now that we have configured the settings and saved it off, how do we handle refreshing the frontend?
			app.GetListOfImages()
		}
	})
	FileMenu.AddSeparator()
	FileMenu.AddText("Quit", keys.CmdOrCtrl("q"), func(_ *menu.CallbackData) {
		runtime.Quit(app.ctx)
	})

	// Create application with options
	err = wails.Run(&options.App{
		Title:      "StableKeepr",
		Width:      s.WindowWidth,
		Height:     s.WindowHeight,
		Fullscreen: s.WindowMaximized,
		Menu:       AppMenu,
		AssetServer: &assetserver.Options{
			Assets:  assets,
			Handler: newLocalAssetHandler(s.ImageDirectories),
		},
		BackgroundColour: &options.RGBA{R: 27, G: 38, B: 54, A: 1},
		OnStartup:        app.startup,
		Bind: []interface{}{
			app,
		},
	})

	if err != nil {
		println("Error:", err.Error())
	}
}

func getSettingsPath() (string, bool) {
	// Get the config file from the user's config folder
	configDir, err := os.UserConfigDir()
	if err != nil {
		println("Error getting user config directory:", err.Error())
		return "", false
	}
	configPath := filepath.Join(configDir, "StableKeepr")
	err = os.MkdirAll(configPath, 0755)
	if err != nil {
		println("Error creating config directory:", err.Error())
		return "", false
	}
	configPath = filepath.Join(configPath, "config.json")
	_, err = os.Stat(configPath)
	doesFileNotExist := os.IsNotExist(err)
	return configPath, doesFileNotExist
}

func getSettings() (settings, error) {
	configPath, doesFileNotExist := getSettingsPath()
	// Does the config file exist?
	if doesFileNotExist {
		// Write the config file
		blankSettings := settings{
			LogLevel:         "info",
			ImageDirectories: []string{},
			WindowHeight:     768,
			WindowWidth:      1024,
			WindowMaximized:  false,
		}
		err := writeSettings(&blankSettings, configPath)
		if err != nil {
			return settings{}, nil
		}
	}
	// Read the config file
	jsonSettings, err := os.ReadFile(configPath)
	if err != nil {
		println("Error reading config file:", err.Error())
		return settings{}, nil
	}

	// Unmarshal the config file
	var s settings
	err = json.Unmarshal(jsonSettings, &s)
	if err != nil {
		println("Error unmarshalling config file:", err.Error())
		return settings{}, nil
	}

	// Resave the settings
	// Do we actually need to do this at this point?
	writeSettings(&s, configPath)
	return s, err
}

func writeSettings(s *settings, configPath string) error {
	// Create the config file
	file, err := os.OpenFile(configPath, os.O_TRUNC|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		println("Error creating config file:", err.Error())
		return err
	}
	// Default the window size to 1024x768
	if s.WindowWidth == 0 {
		s.WindowWidth = 1024
	}
	if s.WindowHeight == 0 {
		s.WindowHeight = 768
	}
	jsonSettings, err := json.MarshalIndent(s, "", "    ")
	if err != nil {
		println("Error marshalling config file:", err.Error())
		return err
	}
	_, err = file.Write(jsonSettings)
	if err != nil {
		println("Error writing config file: ", err.Error())
		return err
	}
	err = file.Close()
	if err != nil {
		println("Error closing config file:", err.Error())
		return err
	}
	return nil
}

// localAssetHandler is a http.HandlerFunc that serves local files from the given directories.
type localAssetHandler struct {
	directories []string
	http.Handler
}

func newLocalAssetHandler(directories []string) *localAssetHandler {
	return &localAssetHandler{
		directories: directories,
	}
}

func (h localAssetHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Get the path from the request
	path := r.URL.Path
	// Get the query string, if any
	query := r.URL.Query()
	fmt.Println("Path: ", path)
	fmt.Println("Query: ", query)
	// Check if the path is a file
	if filepath.Ext(path) != "" {
		// the path has an int in front to specify the directory. get the string up to the second slash and convert to int
		dirIndex, err := strconv.Atoi(strings.Split(path, "/")[1])
		// remove the first directory from the front of the path
		path = strings.Join(strings.Split(path, "/")[2:], "/")
		if err != nil {
			fmt.Println("Could not get path directory index:", err.Error())
			http.NotFound(w, r)
			return
		}
		// Check if the index exists in the directories
		if dirIndex >= len(h.directories) {
			fmt.Println("Directory index out of range:", dirIndex)
			http.NotFound(w, r)
			return
		}
		// Get the directory
		directory := h.directories[dirIndex]
		file := filepath.Join(directory, path)
		// Check if the file exists
		if _, err := os.Stat(file); os.IsNotExist(err) {
			// If the file does not exist, return a 404
			fmt.Println("File not found:", file)
			http.NotFound(w, r)
		} else {
			if width, ok := query["width"]; ok {
				// If the width is specified, resize the image
				width, err := strconv.Atoi(width[0])
				if err != nil {
					fmt.Println("Error parsing width:", err.Error())
					http.NotFound(w, r)
					return
				}
				// Resize the image
				input, err := os.Open(file)
				if err != nil {
					fmt.Println("Error opening image:", err.Error())
					http.NotFound(w, r)
					return
				}
				defer func(input *os.File) {
					err := input.Close()
					if err != nil {
						fmt.Println("Error closing image:", err.Error())
					}
				}(input)
				src, _ := png.Decode(input)
				// calculate the new height while keeping the aspect ratio
				srcWidth := src.Bounds().Max.X
				srcHeight := src.Bounds().Max.Y
				newHeight := int(float64(srcHeight) / float64(srcWidth) * float64(width))
				dst := image.NewRGBA(image.Rect(0, 0, width, newHeight))
				draw.CatmullRom.Scale(dst, dst.Rect, src, src.Bounds(), draw.Over, nil)
				// Encode the image
				buffer := new(bytes.Buffer)
				err = png.Encode(buffer, dst)
				if err != nil {
					fmt.Println("Error encoding image:", err.Error())
					http.NotFound(w, r)
					return
				}
				// Set the content type
				w.Header().Set("Content-Type", "image/png")
				// Set the content length
				w.Header().Set("Content-Length", strconv.Itoa(len(buffer.Bytes())))
				// Write the image
				_, err = w.Write(buffer.Bytes())
				if err != nil {
					fmt.Println("Error writing image:", err.Error())
					http.NotFound(w, r)
					return
				}
				return
			} else {
				// If the width is not specified, serve the image as is
				http.ServeFile(w, r, file)
				return
			}
		}
	}
	// If the path is a directory, return a 404
	http.NotFound(w, r)
}
