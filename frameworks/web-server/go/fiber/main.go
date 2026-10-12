package main

import (
	"github.com/gofiber/fiber/v2"
)

type JsonMessage struct {
	Message string `json:"message"`
}

func main() {
	app := fiber.New(fiber.Config{
		DisableStartupMessage: true,
	})

	app.Get("/healthz", func(c *fiber.Ctx) error {
		return c.SendStatus(fiber.StatusOK)
	})

	app.Get("/plaintext", func(c *fiber.Ctx) error {
		c.Set("Content-Type", "text/plain; charset=utf-8")
		return c.SendString("Hello, World!")
	})

	app.Get("/json", func(c *fiber.Ctx) error {
		return c.JSON(JsonMessage{
			Message: "Hello, World!",
		})
	})

	_ = app.Listen(":8080")
}
