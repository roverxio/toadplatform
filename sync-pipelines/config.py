# read yaml config file
import logging
import os

import structlog
import yaml

from utils.constants import development, production, staging


def read_yaml_config_file(dirname):
    # get environment
    env = os.getenv("ENV", development)
    if env not in [development, production, staging]:
        log.info("Invalid environment. Should be one of development, staging, production")
        exit(1)
    config_file = f"config_{env}.yml"

    try:
        with open(os.path.join(dirname, config_file), "r") as f:
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
