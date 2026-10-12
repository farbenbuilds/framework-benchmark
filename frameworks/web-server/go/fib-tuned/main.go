package main

import (
	"github.com/gofiber/fiber/v2"
)

var (
	plaintextBody = []byte("Hello, World!")
	jsonBody      = []byte("{\"message\": \"Hello, World!\"}")
)

func main() {
	app := fiber.New(fiber.Config{
		Prefork:               false,
		DisableStartupMessage: true,
	})

	app.Get("/healthz", func(c *fiber.Ctx) error {
		return c.SendStatus(fiber.StatusOK)
	})

	app.Get("/plaintext", func(c *fiber.Ctx) error {
		c.Set("Content-Type", "text/plain; charset=utf-8")
		return c.Send(plaintextBody)
	})

	app.Get("/json", func(c *fiber.Ctx) error {
		c.Set("Content-Type", "application/json")
		return c.Send(jsonBody)
	})

	_ = app.Listen(":8080")
}
