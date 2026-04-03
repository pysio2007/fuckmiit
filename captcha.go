package fuckmiit

import (
	"bytes"
	"encoding/base64"
	"fmt"
	"image"
	"image/color"
	_ "image/jpeg"
	_ "image/png"
	"io"
	"strings"
)

const (
	colorTolerance        = 12
	relaxedColorTolerance = 24
	minComponentArea      = 900
	minSideLength         = 24
	topHintAllowance      = 8
	gapApproxSize         = 72
)

var targetColor = color.RGBA{R: 199, G: 186, B: 183, A: 255}

type rect struct {
	left   int
	top    int
	right  int
	bottom int
	area   int
}

func detectSquareBase64WithHint(encoded string, topHint int) (rect, error) {
	imageData, err := decodeBase64Image(encoded)
	if err != nil {
		return rect{}, fmt.Errorf("decode base64 image failed: %w", err)
	}

	return detectSquareFromReaderWithHint(bytes.NewReader(imageData), topHint)
}

func detectSquareFromReaderWithHint(reader io.Reader, topHint int) (rect, error) {
	img, err := decodeImage(reader)
	if err != nil {
		return rect{}, fmt.Errorf("decode image failed: %w", err)
	}

	for _, tolerance := range []int{colorTolerance, relaxedColorTolerance} {
		box, ok := findCaptchaSquare(img, targetColor, tolerance, topHint)
		if ok {
			return box, nil
		}
	}

	if topHint >= 0 {
		return estimateGapFromHint(img, topHint), nil
	}

	return rect{}, fmt.Errorf("square not found")
}

func decodeImage(reader io.Reader) (image.Image, error) {
	img, _, err := image.Decode(reader)
	return img, err
}

func decodeBase64Image(encoded string) ([]byte, error) {
	cleaned := strings.TrimSpace(encoded)
	if comma := strings.IndexByte(cleaned, ','); comma >= 0 {
		cleaned = cleaned[comma+1:]
	}

	if data, err := base64.StdEncoding.DecodeString(cleaned); err == nil {
		return data, nil
	}
	if data, err := base64.RawStdEncoding.DecodeString(cleaned); err == nil {
		return data, nil
	}

	return nil, fmt.Errorf("unsupported base64 image data")
}

func findCaptchaSquare(img image.Image, target color.RGBA, tolerance int, topHint int) (rect, bool) {
	bounds := img.Bounds()
	width := bounds.Dx()
	height := bounds.Dy()
	visited := make([]bool, width*height)
	best := rect{}
	found := false

	for y := bounds.Min.Y; y < bounds.Max.Y; y++ {
		for x := bounds.Min.X; x < bounds.Max.X; x++ {
			index := (y-bounds.Min.Y)*width + (x - bounds.Min.X)
			if visited[index] {
				continue
			}
			visited[index] = true

			if !isNearTarget(img.At(x, y), target, tolerance) {
				continue
			}

			component := floodFill(img, x, y, target, tolerance, visited)
			if !looksLikeSquare(component) || !matchesTopHint(component, topHint) {
				continue
			}

			if !found || component.area > best.area {
				best = component
				found = true
			}
		}
	}

	return best, found
}

func estimateGapFromHint(img image.Image, topHint int) rect {
	bounds := img.Bounds()
	top := clamp(topHint, 0, max(0, bounds.Dy()-gapApproxSize))
	width := bounds.Dx()
	bestLeft := 0
	bestScore := -1

	for left := 0; left <= max(0, width-gapApproxSize); left++ {
		score := windowScore(img, left, top, gapApproxSize)
		if score > bestScore {
			bestScore = score
			bestLeft = left
		}
	}

	return rect{
		left:   bestLeft,
		top:    top,
		right:  min(bounds.Dx()-1, bestLeft+gapApproxSize-1),
		bottom: min(bounds.Dy()-1, top+gapApproxSize-1),
		area:   gapApproxSize * gapApproxSize,
	}
}

func windowScore(img image.Image, left, top, size int) int {
	bounds := img.Bounds()
	score := 0

	for y := top; y < min(bounds.Dy(), top+size); y++ {
		for x := left; x < min(bounds.Dx(), left+size); x++ {
			score += gapPixelScore(img.At(bounds.Min.X+x, bounds.Min.Y+y))
		}
	}

	return score
}

func gapPixelScore(c color.Color) int {
	r16, g16, b16, _ := c.RGBA()
	r := int(uint8(r16 >> 8))
	g := int(uint8(g16 >> 8))
	b := int(uint8(b16 >> 8))
	maxChannel := max(r, max(g, b))
	minChannel := min(r, min(g, b))
	spread := maxChannel - minChannel
	avg := (r + g + b) / 3

	if avg < 100 || avg > 235 {
		return 0
	}

	score := max(0, 45-spread*2)
	score += max(0, 30-abs(avg-189))
	if channelDistance(uint8(r), targetColor.R) <= relaxedColorTolerance &&
		channelDistance(uint8(g), targetColor.G) <= relaxedColorTolerance &&
		channelDistance(uint8(b), targetColor.B) <= relaxedColorTolerance {
		score += 25
	}

	return score
}

func floodFill(img image.Image, startX, startY int, target color.RGBA, tolerance int, visited []bool) rect {
	bounds := img.Bounds()
	width := bounds.Dx()
	queueX := []int{startX}
	queueY := []int{startY}
	component := rect{
		left:   startX - bounds.Min.X,
		top:    startY - bounds.Min.Y,
		right:  startX - bounds.Min.X,
		bottom: startY - bounds.Min.Y,
		area:   0,
	}

	for head := 0; head < len(queueX); head++ {
		x := queueX[head]
		y := queueY[head]
		localX := x - bounds.Min.X
		localY := y - bounds.Min.Y

		component.area++
		if localX < component.left {
			component.left = localX
		}
		if localX > component.right {
			component.right = localX
		}
		if localY < component.top {
			component.top = localY
		}
		if localY > component.bottom {
			component.bottom = localY
		}

		for _, step := range [][2]int{{-1, 0}, {1, 0}, {0, -1}, {0, 1}} {
			nextX := x + step[0]
			nextY := y + step[1]
			if nextX < bounds.Min.X || nextX >= bounds.Max.X || nextY < bounds.Min.Y || nextY >= bounds.Max.Y {
				continue
			}

			nextIndex := (nextY-bounds.Min.Y)*width + (nextX - bounds.Min.X)
			if visited[nextIndex] {
				continue
			}
			visited[nextIndex] = true

			if !isNearTarget(img.At(nextX, nextY), target, tolerance) {
				continue
			}

			queueX = append(queueX, nextX)
			queueY = append(queueY, nextY)
		}
	}

	return component
}

func looksLikeSquare(box rect) bool {
	width := box.right - box.left + 1
	height := box.bottom - box.top + 1
	if box.area < minComponentArea || width < minSideLength || height < minSideLength {
		return false
	}

	if width > height {
		return width-height <= width/3
	}

	return height-width <= height/3
}

func matchesTopHint(box rect, topHint int) bool {
	if topHint < 0 {
		return true
	}

	return abs(box.top-topHint) <= topHintAllowance
}

func isNearTarget(c color.Color, target color.RGBA, tolerance int) bool {
	r, g, b, _ := c.RGBA()
	return channelDistance(uint8(r>>8), target.R) <= tolerance &&
		channelDistance(uint8(g>>8), target.G) <= tolerance &&
		channelDistance(uint8(b>>8), target.B) <= tolerance
}

func channelDistance(a, b uint8) int {
	if a > b {
		return int(a - b)
	}
	return int(b - a)
}

func abs(v int) int {
	if v < 0 {
		return -v
	}
	return v
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}

func clamp(v, low, high int) int {
	if v < low {
		return low
	}
	if v > high {
		return high
	}
	return v
}
