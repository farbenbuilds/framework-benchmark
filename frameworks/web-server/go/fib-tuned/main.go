package main

import (
	"github.com/gofiber/fiber/v3"
)

var (
	plaintextBody = []byte("Hello, World!")
	jsonBody      = []byte("{\"message\": \"Hello, World!\"}")
)

func main() {
	app := fiber.New()

	app.Get("/healthz", func(c fiber.Ctx) error {
		return c.Send(nil)
	})

	app.Get("/plaintext", func(c fiber.Ctx) error {
		c.Set("Content-Type", "text/plain; charset=utf-8")
		return c.Send(plaintextBody)
	})

	app.Get("/json", func(c fiber.Ctx) error {
		c.Set("Content-Type", "application/json")
		return c.Send(jsonBody)
	})

	_ = app.Listen(":8080", fiber.ListenConfig{
		EnablePrefork:         false,
		DisableStartupMessage: true,
	})
}
