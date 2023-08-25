# read yaml config file
import logging
import os

import structlog
import yaml


def read_yaml_config_file(dirname):
    try:
        with open(os.path.join(dirname, "config.yml"), "r") as f:
            configuration = yaml.safe_load(f)
        return configuration
    except FileNotFoundError:
        log.info("Config file not found")
        exit(1)


def init_logging():
    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            structlog.processors.StackInfoRenderer(),
            structlog.dev.set_exc_info,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.JSONRenderer(),
        ],
        wrapper_class=structlog.make_filtering_bound_logger(logging.NOTSET),
        context_class=dict,
        logger_factory=structlog.PrintLoggerFactory(),
        cache_logger_on_first_use=False,
    )
    logger = structlog.get_logger()
    return logger


log = init_logging()
config = read_yaml_config_file(os.path.dirname(os.path.realpath(__file__)))
