// layanan-integrasi/config/loader.go

package config

import (
	"io/ioutil"
	"os"
	"strconv"

	"gopkg.in/yaml.v3"
)

var AppConfig *Config

type Config struct {
	Server struct {
		Port int    `yaml:"port"`
		Mode string `yaml:"mode"`
	} `yaml:"server"`
	Database struct {
		Host     string `yaml:"host"`
		Port     int    `yaml:"port"`
		User     string `yaml:"user"`
		Password string `yaml:"password"`
		Name     string `yaml:"name"`
	} `yaml:"database"`
	MySimkari struct {
		URL   string `yaml:"url"`
		Token string `yaml:"token"`
	} `yaml:"mysimkari"`
}

type DatabaseConfig struct {
	Host     string
	Port     int
	User     string
	Password string
	Name     string
}

func LoadConfig(path string) (*Config, error) {
	data, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, err
	}

	// Override DATABASE dari ENV
	if val := os.Getenv("DB_HOST"); val != "" {
		cfg.Database.Host = val
	}
	if val := os.Getenv("DB_PORT"); val != "" {
		if port, err := strconv.Atoi(val); err == nil {
			cfg.Database.Port = port
		}
	}
	if val := os.Getenv("DB_USER"); val != "" {
		cfg.Database.User = val
	}
	if val := os.Getenv("DB_PASSWORD"); val != "" {
		cfg.Database.Password = val
	}
	if val := os.Getenv("DB_NAME"); val != "" {
		cfg.Database.Name = val
	}

	// Override URL MySimkari
	if val := os.Getenv("URL_MYSIMKARI"); val != "" {
		cfg.MySimkari.URL = val
	}

	// Override token MySimkari
	if val := os.Getenv("TOKEN_MYSIMKARI"); val != "" {
		cfg.MySimkari.Token = val
	}

	return &cfg, nil
}

func (c *Config) GetDatabaseConfig() DatabaseConfig {
	return DatabaseConfig{
		Host:     c.Database.Host,
		Port:     c.Database.Port,
		User:     c.Database.User,
		Password: c.Database.Password,
		Name:     c.Database.Name,
	}
}

func Load() {
	cfg, err := LoadConfig("config/config.yaml")
	if err != nil {
		panic("Gagal load config: " + err.Error())
	}
	AppConfig = cfg
}
