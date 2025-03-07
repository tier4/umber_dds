#include "dds/dds.h"
#include "Shape.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>

/* An array of one message (aka sample in dds terms) will be used. */
#define MAX_SAMPLES 1

int main (int argc, char ** argv)
{
  dds_entity_t participant;
  dds_entity_t topic;
  dds_entity_t reader;
  ShapeType *msg;
  void *samples[MAX_SAMPLES];
  dds_sample_info_t infos[MAX_SAMPLES];
  dds_return_t rc;
  dds_qos_t *qos;
  time_t start_time;
  uint32_t receiv_count;
  (void)argc;
  (void)argv;

  printf("--- shapes_demo_cyclonedds/subscriber start\n");

  /* Create a Participant. */
  participant = dds_create_participant (DDS_DOMAIN_DEFAULT, NULL, NULL);
  if (participant < 0)
    DDS_FATAL("dds_create_participant: %s\n", dds_strretcode(-participant));

  /* Create a Topic. */
  topic = dds_create_topic (
    participant, &ShapeType_desc, "Square", NULL, NULL);
  if (topic < 0)
    DDS_FATAL("dds_create_topic: %s\n", dds_strretcode(-topic));

  /* Create a besteffort Reader. */
  qos = dds_create_qos ();
  dds_qset_reliability (qos, DDS_RELIABILITY_BEST_EFFORT, DDS_SECS (10));
  reader = dds_create_reader (participant, topic, qos, NULL);
  if (reader < 0)
    DDS_FATAL("dds_create_reader: %s\n", dds_strretcode(-reader));
  dds_delete_qos(qos);

  printf ("\n=== [Subscriber] Waiting for a sample ...\n");
  fflush (stdout);

  /* Initialize sample buffer, by pointing the void pointer within
   * the buffer array to a valid sample memory location. */
  samples[0] = ShapeType__alloc ();

  start_time = time(NULL);
  if (start_time == (time_t)-1) {
    printf ("get time failed");
    return -1;
  }

  receiv_count = 0;

  /* Poll until data has been read. */
  while (true)
  {
    if (difftime(time(NULL), start_time) > 30.0) {
      break;
    }
    if (receiv_count > 5) {
      break;
    }
    /* Do the actual read.
     * The return value contains the number of read samples. */
    rc = dds_take (reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
    if (rc < 0) {
      DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));
      break;
    }

    /* Check if we read some data and it is valid. */
    if ((rc > 0) && (infos[0].valid_data))
    {
      /* Print Message. */
      msg = (ShapeType*) samples[0];
      printf ("=== [Subscriber] Received : ");
      printf ("Message (%s, x:%"PRId32", y:%"PRId32", shapesize:%"PRId32")\n", msg->color, msg->x, msg->y, msg->shapesize);
      fflush (stdout);
      receiv_count++;
      // break;
    }
    else
    {
      /* Polling sleep. */
      dds_sleepfor (DDS_MSECS (20));
    }

  }

  /* Free the data location. */
  ShapeType_free (samples[0], DDS_FREE_ALL);

  /* Deleting the participant will delete all its children recursively as well. */
  rc = dds_delete (participant);
  if (rc != DDS_RETCODE_OK)
    DDS_FATAL("dds_delete: %s\n", dds_strretcode(-rc));

  printf("--- shapes_demo_cyclonedds/subscriber end\n");
  if (receiv_count > 5) {
    return EXIT_SUCCESS;
  } else {
    return -1;
  }
}
